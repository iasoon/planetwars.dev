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
use crate::match_log::{MatchLogMessage, MatchLogger, StdErrMessage};
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
        match_logger: MatchLogger,
    ) -> Box<dyn PlayerHandle> {
        let process = spawn_docker_process(self).await.unwrap();
        let (handle, runner) = create_docker_bot(process, player_id, event_bus, match_logger);
        tokio::spawn(runner.run());
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
                // setting this to true causes duplicate error output. Why?
                logs: Some(false),
                ..Default::default()
            }),
        )
        .await?;

    Ok(ContainerProcess {
        docker,
        container_id,
        stdin: input,
        output,
    })
}

struct ContainerProcess {
    docker: Docker,
    container_id: String,
    stdin: Pin<Box<dyn AsyncWrite + Send>>,
    output: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>,
}

impl ContainerProcess {
    // &mut is required here to make terminate().await Sync
    async fn terminate(&mut self) -> Result<(), bollard::errors::Error> {
        self.docker
            .remove_container(
                &self.container_id,
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
    }
}

fn create_docker_bot(
    process: ContainerProcess,
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    match_logger: MatchLogger,
) -> (DockerBotHandle, DockerBotRunner) {
    let (tx, rx) = mpsc::unbounded_channel();
    let bot_handle = DockerBotHandle { tx };
    let bot_runner = DockerBotRunner {
        process,
        player_id,
        event_bus,
        match_logger,
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
    process: ContainerProcess,
    event_bus: Arc<Mutex<EventBus>>,
    rx: mpsc::UnboundedReceiver<RequestMessage>,
    match_logger: MatchLogger,
    player_id: u32,
}

impl DockerBotRunner {
    pub async fn run(mut self) {
        while let Some(request) = self.rx.recv().await {
            let resp_fut = self.communicate(&request.content);
            let result = timeout(request.timeout, resp_fut).await;
            let request_response = match result {
                Ok(Ok(response)) => Ok(response.to_vec()),
                // this one happens when a bot output stream ends, map this to Timeout for now
                Ok(Err(_read_error)) => Err(RequestError::Timeout),
                Err(_elapsed) => Err(RequestError::Timeout),
            };
            let request_id = (self.player_id, request.request_id);

            self.event_bus
                .lock()
                .unwrap()
                .resolve_request(request_id, request_response);
        }

        self.process
            .terminate()
            .await
            .expect("could not terminate process");
    }

    pub async fn communicate(&mut self, input: &[u8]) -> io::Result<Bytes> {
        self.write_line(input).await?;
        self.read_line().await
    }

    async fn write_line(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.process.stdin.write_all(bytes).await?;
        self.process.stdin.write_u8(b'\n').await?;
        self.process.stdin.flush().await?;
        Ok(())
    }

    async fn read_line(&mut self) -> io::Result<Bytes> {
        while let Some(item) = self.process.output.next().await {
            let log_output = item.expect("failed to get log output");
            match log_output {
                LogOutput::StdOut { message } => {
                    // TODO: this is not correct (buffering and such)
                    return Ok(message);
                }
                LogOutput::StdErr { mut message } => {
                    // TODO
                    if message.ends_with(b"\n") {
                        message.truncate(message.len() - 1);
                    }
                    for line in message.split(|c| *c == b'\n') {
                        let message = StdErrMessage {
                            player_id: self.player_id,
                            message: String::from_utf8_lossy(line).to_string(),
                        };
                        self.match_logger
                            .send(MatchLogMessage::StdErr(message))
                            .unwrap();
                    }
                }
                _ => (),
            }
        }

        // at this point the stream has ended
        // does this mean the container has exited?

        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "no response received",
        ))
    }
}
