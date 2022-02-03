use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bollard::container::{self, AttachContainerOptions, AttachContainerResults, LogOutput};
use bollard::Docker;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::match_context::{EventBus, PlayerHandle, RequestError, RequestMessage};
use crate::BotSpec;

#[derive(Clone, Debug)]
pub struct DockerBotSpec {
    pub image: String,
    pub code_path: PathBuf,
    pub argv: Vec<String>,
}

#[async_trait]
impl BotSpec for DockerBotSpec {
    async fn run_bot(
        &self,
        player_id: u32,
        event_bus: Arc<Mutex<EventBus>>,
    ) -> Box<dyn PlayerHandle> {
        let (handle, runner) = create_docker_bot(player_id, event_bus);
        let process = spawn_docker_process(self).await.unwrap();
        tokio::spawn(runner.run(process));
        return Box::new(handle);
    }
}

async fn spawn_docker_process(
    params: &DockerBotSpec,
) -> Result<ContainerProcess, bollard::errors::Error> {
    let docker = Docker::connect_with_socket_defaults()?;
    let bot_code_dir = std::fs::canonicalize(&params.code_path).unwrap();
    let code_dir_str = bot_code_dir.as_os_str().to_str().unwrap();

    let config = container::Config {
        image: Some(params.image.clone()),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(vec![format!("{}:{}", code_dir_str, "/workdir")]),
            ..Default::default()
        }),
        working_dir: Some("/workdir".to_string()),
        cmd: Some(params.argv.clone()),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
        ..Default::default()
    };

    let response = docker
        .create_container::<&str, String>(None, config)
        .await?;
    let container_id = response.id;

    docker
        .start_container::<String>(&container_id, None)
        .await?;

    let AttachContainerResults { output, input } = docker
        .attach_container(
            &container_id,
            Some(AttachContainerOptions::<String> {
                stdout: Some(true),
                stderr: Some(true),
                stdin: Some(true),
                stream: Some(true),
                logs: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    Ok(ContainerProcess {
        stdin: input,
        output,
    })
}

pub struct ContainerProcess {
    stdin: Pin<Box<dyn AsyncWrite + Send>>,
    output: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>,
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
                    println!("{}", String::from_utf8_lossy(&message));
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

fn create_docker_bot(
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
) -> (DockerBotHandle, DockerBotRunner) {
    let (tx, rx) = mpsc::unbounded_channel();
    let bot_handle = DockerBotHandle { tx };
    let bot_runner = DockerBotRunner {
        player_id,
        event_bus,
        rx,
    };
    (bot_handle, bot_runner)
}

pub struct DockerBotHandle {
    tx: mpsc::UnboundedSender<RequestMessage>,
}

impl PlayerHandle for DockerBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        self.tx
            .send(r)
            .expect("failed to send message to local bot");
    }
}

pub struct DockerBotRunner {
    event_bus: Arc<Mutex<EventBus>>,
    rx: mpsc::UnboundedReceiver<RequestMessage>,
    player_id: u32,
}

impl DockerBotRunner {
    pub async fn run(mut self, mut process: ContainerProcess) {
        while let Some(request) = self.rx.recv().await {
            let resp_fut = process.communicate(&request.content);
            let result = timeout(request.timeout, resp_fut)
                .await
                // TODO: how can this failure be handled cleanly?
                .expect("process read failed");
            let result = match result {
                Ok(line) => Ok(line.to_vec()),
                Err(_elapsed) => Err(RequestError::Timeout),
            };
            let request_id = (self.player_id, request.request_id);

            self.event_bus
                .lock()
                .unwrap()
                .resolve_request(request_id, result);
        }
    }
}
