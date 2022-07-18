pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use pb::bot_api_service_client::BotApiServiceClient;
use planetwars_matchrunner::bot_runner::Bot;
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{metadata::MetadataValue, transport::Channel, Request, Status};

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

    let created_match = create_match(channel.clone()).await.unwrap();
    run_player(bot_config, created_match.player_key, channel).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn create_match(channel: Channel) -> Result<pb::CreatedMatch, Status> {
    let mut client = BotApiServiceClient::new(channel);
    let res = client
        .create_match(Request::new(pb::MatchRequest {
            opponent_name: "simplebot".to_string(),
        }))
        .await;
    res.map(|response| response.into_inner())
}

async fn run_player(bot_config: BotConfig, player_key: String, channel: Channel) {
    let mut client = BotApiServiceClient::with_interceptor(channel, |mut req: Request<()>| {
        let player_key: MetadataValue<_> = player_key.parse().unwrap();
        req.metadata_mut().insert("player_key", player_key);
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
        let moves = bot_process.communicate(&message.content).await.unwrap();
        tx.send(pb::PlayerRequestResponse {
            request_id: message.request_id,
            content: moves.as_bytes().to_vec(),
        })
        .unwrap();
    }
}
