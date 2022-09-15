use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};

use planetwars_rules::protocol::State;
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MatchLogMessage {
    #[serde(rename = "gamestate")]
    GameState(State),
    #[serde(rename = "stderr")]
    StdErr(StdErrMessage),
    #[serde(rename = "bad_command")]
    BadCommand {
        player_id: u32,
        command: String,
        error: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StdErrMessage {
    pub player_id: u32,
    pub message: String,
}

pub type MatchLogger = mpsc::UnboundedSender<MatchLogMessage>;

pub async fn create_log_sink(log_file_path: &Path) -> MatchLogger {
    let (tx, rx) = mpsc::unbounded_channel();
    let log_file = File::create(log_file_path)
        .await
        .expect("Could not create log file");
    tokio::spawn(run_log_sink(rx, log_file));
    tx
}

async fn run_log_sink(mut rx: mpsc::UnboundedReceiver<MatchLogMessage>, mut file: File) {
    while let Some(message) = rx.recv().await {
        let json = serde_json::to_string(&message).expect("failed to serialize message");
        file.write_all(json.as_bytes())
            .await
            .expect("failed to write log message to file");
        file.write_all(b"\n")
            .await
            .expect("failed to write newline log message to file");
    }
}
