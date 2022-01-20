extern crate planetwars_matchrunner;
extern crate tokio;

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use bollard::container::{self, LogOutput};
use bollard::exec::StartExecResults;
use bollard::Docker;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use planetwars_matchrunner::{
    match_context::{EventBus, MatchCtx, PlayerHandle, RequestMessage},
    pw_match, MatchConfig, MatchMeta, PlayerInfo,
};
use planetwars_rules::protocol as proto;
use std::env;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;

const IMAGE: &'static str = "python:3.10-slim-buster";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);
    let map_path = args[1].clone();
    _run_match(map_path).await;
}

async fn _run_match(map_path: String) {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    create_player_process(&docker).await.unwrap();
}

async fn create_player_process(docker: &Docker) -> Result<(), bollard::errors::Error> {
    let bot_code_dir = "../simplebot";
    let config = container::Config {
        image: Some(IMAGE),
        host_config: Some(container::HostConfig {
            binds: Some(vec![format!("{}:{}", bot_code_dir, "/workdir")]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let response = docker.create_container::<&str, &str>(None, config).await?;
    let container_id = response.id;

    docker
        .start_container::<String>(&container_id, None)
        .await?;

    let exec_id = docker
        .create_exec::<&str>(
            &container_id,
            bollard::exec::CreateExecOptions {
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                working_dir: Some("/workdir"),
                cmd: Some(vec!["python", "simplebot.py"]),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    let start_exec_results = docker.start_exec(&exec_id, None).await?;
    let mut process = match start_exec_results {
        StartExecResults::Detached => panic!("failed to get io channels"),
        StartExecResults::Attached { input, output } => ContainerProcess {
            stdin: input,
            output,
        },
    };

    let state = proto::State {
        planets: vec![
            proto::Planet {
                name: "a".to_string(),
                owner: Some(1),
                ship_count: 100,
                x: -1.0,
                y: 0.0,
            },
            proto::Planet {
                name: "b".to_string(),
                owner: Some(2),
                ship_count: 100,
                x: 1.0,
                y: 0.0,
            },
        ],
        expeditions: vec![],
    };

    let serialized = serde_json::to_vec(&state).unwrap();
    let out = process.communicate(&serialized).await?;

    print!("{}", String::from_utf8(out.to_vec()).unwrap());

    Ok(())
}

pub struct ContainerProcess {
    stdin: Pin<Box<dyn AsyncWrite + Send>>,
    output: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>>>>,
}

impl ContainerProcess {
    pub async fn communicate(&mut self, input: &[u8]) -> io::Result<Bytes> {
        self.write_line(input).await?;
        self.read_line().await
    }

    async fn write_line(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stdin.write_all(bytes).await?;
        self.stdin.write_u8(b'\n').await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_line(&mut self) -> io::Result<Bytes> {
        while let Some(item) = self.output.next().await {
            let log_output = item.expect("failed to get log output");
            match log_output {
                LogOutput::StdOut { message } => {
                    // TODO: this is not correct (buffering and such)
                    return Ok(message);
                }
                LogOutput::StdErr { message } => {
                    // TODO
                    println!("stderr: {}", String::from_utf8_lossy(&message));
                }
                _ => (),
            }
        }

        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "no response received",
        ))
    }
}
