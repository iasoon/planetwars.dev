pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use clap::Parser;
use pb::bot_api_service_client::BotApiServiceClient;
use planetwars_matchrunner::bot_runner::Bot;
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{metadata::MetadataValue, transport::Channel, Request, Status};

#[derive(clap::Parser)]
struct PlayMatch {
    #[clap(value_parser)]
    bot_config_path: String,

    #[clap(value_parser)]
    opponent_name: String,

    #[clap(
        value_parser,
        long,
        default_value = "https://planetwars.dev:7492",
        env = "PLANETWARS_GRPC_SERVER_URL"
    )]
    grpc_server_url: String,
}

#[derive(Deserialize)]
struct BotConfig {
    #[allow(dead_code)]
    name: String,
    command: Command,
    working_directory: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Command {
    String(String),
    Argv(Vec<String>),
}

impl Command {
    pub fn to_argv(&self) -> Vec<String> {
        match self {
            Command::Argv(vec) => vec.clone(),
            Command::String(s) => shlex::split(s).expect("invalid command string"),
        }
    }
}

#[tokio::main]
async fn main() {
    let play_match = PlayMatch::parse();

    let content = std::fs::read_to_string(play_match.bot_config_path).unwrap();
    let bot_config: BotConfig = toml::from_str(&content).unwrap();

    let uri = play_match
        .grpc_server_url
        .parse()
        .expect("invalid grpc url");

    let channel = Channel::builder(uri).connect().await.unwrap();

    let created_match = create_match(channel.clone(), play_match.opponent_name)
        .await
        .unwrap();
    run_player(bot_config, created_match.player_key, channel).await;
    println!(
        "Match completed. Watch the replay at {}",
        created_match.match_url
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn create_match(channel: Channel, opponent_name: String) -> Result<pb::CreatedMatch, Status> {
    let mut client = BotApiServiceClient::new(channel);
    let res = client
        .create_match(Request::new(pb::MatchRequest { opponent_name }))
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
        working_dir: PathBuf::from(
            bot_config
                .working_directory
                .unwrap_or_else(|| ".".to_string()),
        ),
        argv: bot_config.command.to_argv(),
    }
    .spawn_process();

    let (tx, rx) = mpsc::unbounded_channel();
    let mut stream = client
        .connect_player(UnboundedReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();
    while let Some(message) = stream.message().await.unwrap() {
        use pb::client_message::ClientMessage;
        use pb::server_message::ServerMessage;

        match message.server_message {
            Some(ServerMessage::PlayerRequest(req)) => {
                let moves = bot_process.communicate(&req.content).await.unwrap();
                let resp = pb::PlayerRequestResponse {
                    request_id: req.request_id,
                    content: moves.as_bytes().to_vec(),
                };
                let msg = pb::ClientMessage {
                    client_message: Some(ClientMessage::RequestResponse(resp)),
                };
                tx.send(msg).unwrap();
            }
            _ => {} // pass
        }
    }
}
