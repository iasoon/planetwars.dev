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
use futures::{Stream, StreamExt};
use planetwars_matchrunner::{
    match_context::{EventBus, MatchCtx, PlayerHandle},
    pw_match, MatchConfig, MatchMeta, PlayerInfo,
};
use planetwars_rules::protocol as proto;
use planetwars_rules::PwConfig;
use std::env;
use tokio::io::{AsyncWrite, AsyncWriteExt};

const IMAGE: &'static str = "simplebot:latest";

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
    let config = container::Config {
        image: Some(IMAGE),
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
                cmd: Some(vec!["python", "simplebot.py"]),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    let start_exec_results = docker.start_exec(&exec_id, None).await?;
    let (mut input, mut output) = match start_exec_results {
        StartExecResults::Detached => panic!("failed to get io channels"),
        StartExecResults::Attached { input, output } => (input, output),
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
    input.write_all(&serialized).await?;
    input.write(b"\n").await?;
    input.flush().await?;

    while let Some(item) = output.next().await {
        let log_output = item.expect("failed to get log output");
        match log_output {
            LogOutput::StdOut { message } => {
                println!("stdout: {}", String::from_utf8_lossy(&message));
            }
            LogOutput::StdErr { message } => {
                println!("stderr: {}", String::from_utf8_lossy(&message));
            }
            _ => (),
        }
    }

    Ok(())
}
