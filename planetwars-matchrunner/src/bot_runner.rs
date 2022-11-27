use std::io;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use super::match_context::EventBus;
use super::match_context::PlayerHandle;
use super::match_context::RequestError;
use super::match_context::RequestMessage;
// TODO: this is exactly the same as the docker bot handle.
// should this abstraction be removed?
pub struct LocalBotHandle {
    tx: mpsc::UnboundedSender<RequestMessage>,
    join_handle: JoinHandle<()>,
}

impl PlayerHandle for LocalBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        self.tx
            .send(r)
            .expect("failed to send message to local bot");
    }

    fn into_join_handle(self: Box<Self>) -> JoinHandle<()> {
        self.join_handle
    }
}

pub fn run_local_bot(player_id: u32, event_bus: Arc<Mutex<EventBus>>, bot: Bot) -> LocalBotHandle {
    let (tx, rx) = mpsc::unbounded_channel();

    let runner = LocalBotRunner {
        event_bus,
        rx,
        player_id,
        bot,
    };
    let join_handle = tokio::spawn(runner.run());

    LocalBotHandle { tx, join_handle }
}

pub struct LocalBotRunner {
    event_bus: Arc<Mutex<EventBus>>,
    rx: mpsc::UnboundedReceiver<RequestMessage>,
    player_id: u32,
    bot: Bot,
}

impl LocalBotRunner {
    pub async fn run(mut self) {
        let mut process = self.bot.spawn_process();

        while let Some(request) = self.rx.recv().await {
            let resp_fut = process.communicate(&request.content);
            let result = timeout(request.timeout, resp_fut)
                .await
                // TODO: how can this failure be handled cleanly?
                .expect("process read failed");
            let result = match result {
                Ok(line) => Ok(line.into_bytes()),
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

#[derive(Debug, Clone)]
pub struct Bot {
    pub working_dir: PathBuf,
    pub argv: Vec<String>,
}

impl Bot {
    pub fn spawn_process(&self) -> BotProcess {
        let mut child = process::Command::new(&self.argv[0])
            .args(&self.argv[1..])
            .current_dir(self.working_dir.clone())
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawning failed");

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout).lines();

        BotProcess {
            stdin: child.stdin.take().unwrap(),
            stdout: reader,
            child,
        }
    }
}

pub struct BotProcess {
    #[allow(dead_code)]
    pub child: process::Child,
    pub stdin: process::ChildStdin,
    pub stdout: Lines<BufReader<process::ChildStdout>>,
}

impl BotProcess {
    // TODO: gracefully handle errors
    pub async fn communicate(&mut self, input: &[u8]) -> io::Result<String> {
        self.stdin.write_all(input).await?;
        self.stdin.write_u8(b'\n').await?;
        let line = self.stdout.next_line().await?;
        line.ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "no response received"))
    }
}
