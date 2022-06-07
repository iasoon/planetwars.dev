pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use pb::bot_api_service_client::BotApiServiceClient;
use planetwars_matchrunner::bot_runner::Bot;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{metadata::MetadataValue, transport::Channel, Request};

#[derive(Deserialize)]
struct BotConfig {
    #[allow(dead_code)]
    name: String,
    command: Vec<String>,
}

#[tokio::main]
async fn main() {
    let content = std::fs::read_to_string("simplebot.toml").unwrap();
    let bot_config: BotConfig = toml::from_str(&content).unwrap();

    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await
        .unwrap();

    let mut client = BotApiServiceClient::with_interceptor(channel, |mut req: Request<()>| {
        let player_id: MetadataValue<_> = "test_player".parse().unwrap();
        req.metadata_mut().insert("player_id", player_id);
        Ok(req)
    });

    let mut bot_process = Bot {
        working_dir: PathBuf::from("."),
        argv: bot_config.command,
    }
    .spawn_process();

    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = client
        .connect_bot(UnboundedReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();
    while let Some(message) = stream.message().await.unwrap() {
        let state = std::str::from_utf8(&message.content).unwrap();
        let moves = bot_process.communicate(&message.content).await.unwrap();
        tx.send(pb::PlayerRequestResponse {
            request_id: message.request_id,
            content: moves.as_bytes().to_vec(),
        })
        .unwrap();
    }
    std::mem::drop(tx);
    // for clean exit
    std::mem::drop(client);
}
