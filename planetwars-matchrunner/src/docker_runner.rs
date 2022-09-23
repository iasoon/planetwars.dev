use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bollard::container::{self, AttachContainerOptions, AttachContainerResults, LogOutput};
use bollard::Docker;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::match_context::{EventBus, PlayerHandle, RequestError, RequestMessage};
use crate::match_log::{MatchLogMessage, MatchLogger, StdErrMessage};
use crate::BotSpec;

// TODO: this API needs a better design with respect to pulling
// and general container management
#[derive(Clone, Debug)]
pub struct DockerBotSpec {
    pub image: String,
    pub binds: Option<Vec<String>>,
    pub argv: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub pull: bool,
    pub credentials: Option<Credentials>,
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
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
        let handle = run_docker_bot(process, player_id, event_bus, match_logger);
        return Box::new(handle);
    }
}

async fn spawn_docker_process(
    params: &DockerBotSpec,
) -> Result<ContainerProcess, bollard::errors::Error> {
    let docker = Docker::connect_with_socket_defaults()?;

    if params.pull {
        let mut create_image_stream = docker.create_image(
            Some(bollard::image::CreateImageOptions {
                from_image: params.image.as_str(),
                ..Default::default()
            }),
            None,
            params
                .credentials
                .as_ref()
                .map(|credentials| bollard::auth::DockerCredentials {
                    username: Some(credentials.username.clone()),
                    password: Some(credentials.password.clone()),
                    ..Default::default()
                }),
        );

        while let Some(item) = create_image_stream.next().await {
            // just consume the stream for now,
            // and make noise when something breaks
            let _info = item.expect("hit error in docker pull");
        }
    }

    let memory_limit = 512 * 1024 * 1024; // 512MB
    let config = container::Config {
        image: Some(params.image.clone()),
        host_config: Some(bollard::models::HostConfig {
            binds: params.binds.clone(),
            network_mode: Some("none".to_string()),
            memory: Some(memory_limit),
            memory_swap: Some(memory_limit),
            // TODO: this seems to have caused weird delays when executing bots
            // on the production server. A solution should still be found, though.
            // cpu_period: Some(100_000),
            // cpu_quota: Some(10_000),
            ..Default::default()
        }),
        working_dir: params.working_dir.clone(),
        cmd: params.argv.clone(),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
        network_disabled: Some(true),
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

fn run_docker_bot(
    process: ContainerProcess,
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    match_logger: MatchLogger,
) -> DockerBotHandle {
    let (tx, rx) = mpsc::unbounded_channel();
    let bot_runner = DockerBotRunner {
        process,
        player_id,
        event_bus,
        match_logger,
        rx,
    };

    let join_handle = tokio::spawn(bot_runner.run());

    DockerBotHandle { tx, join_handle }
}

pub struct DockerBotHandle {
    tx: mpsc::UnboundedSender<RequestMessage>,
    join_handle: JoinHandle<()>,
}

impl PlayerHandle for DockerBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        self.tx
            .send(r)
            .expect("failed to send message to local bot");
    }

    fn into_join_handle(self: Box<Self>) -> JoinHandle<()> {
        self.join_handle
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
