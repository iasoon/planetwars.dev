pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use runner::match_context::{EventBus, PlayerHandle, RequestMessage};
use runner::match_log::MatchLogger;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};

use planetwars_matchrunner as runner;

use crate::db;
use crate::{ConnectionPool, MAPS_DIR, MATCHES_DIR};

use super::matches::code_bundle_to_botspec;

pub struct BotApiServer {
    router: PlayerRouter,
}

/// Routes players to their handler
#[derive(Clone)]
struct PlayerRouter {
    routing_table: Arc<Mutex<HashMap<String, SyncThingData>>>,
}

impl PlayerRouter {
    pub fn new() -> Self {
        PlayerRouter {
            routing_table: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// TODO: implement a way to expire entries
impl PlayerRouter {
    fn put(&self, player_id: String, entry: SyncThingData) {
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.insert(player_id, entry);
    }

    fn get(&self, player_id: &str) -> Option<SyncThingData> {
        // TODO: this design does not allow for reconnects. Is this desired?
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.remove(player_id)
    }
}

#[tonic::async_trait]
impl pb::bot_api_service_server::BotApiService for BotApiServer {
    type ConnectBotStream = UnboundedReceiverStream<Result<pb::PlayerRequest, Status>>;

    async fn connect_bot(
        &self,
        req: Request<Streaming<pb::PlayerRequestResponse>>,
    ) -> Result<Response<Self::ConnectBotStream>, Status> {
        // TODO: clean up errors
        let player_id = req
            .metadata()
            .get("player_id")
            .ok_or_else(|| Status::unauthenticated("no player_id provided"))?;

        let player_id_str = player_id
            .to_str()
            .map_err(|_| Status::invalid_argument("unreadable string"))?;

        let sync_data = self
            .router
            .get(player_id_str)
            .ok_or_else(|| Status::not_found("player_id not found"))?;

        let stream = req.into_inner();

        sync_data.tx.send(stream).unwrap();
        Ok(Response::new(UnboundedReceiverStream::new(
            sync_data.server_messages,
        )))
    }
}

struct SyncThingData {
    tx: oneshot::Sender<Streaming<pb::PlayerRequestResponse>>,
    server_messages: mpsc::UnboundedReceiver<Result<pb::PlayerRequest, Status>>,
}

struct RemoteBotSpec {
    router: PlayerRouter,
}

#[tonic::async_trait]
impl runner::BotSpec for RemoteBotSpec {
    async fn run_bot(
        &self,
        player_id: u32,
        event_bus: Arc<Mutex<EventBus>>,
        _match_logger: MatchLogger,
    ) -> Box<dyn PlayerHandle> {
        let (tx, rx) = oneshot::channel();
        let (server_msg_snd, server_msg_recv) = mpsc::unbounded_channel();
        self.router.put(
            "test_player".to_string(),
            SyncThingData {
                tx,
                server_messages: server_msg_recv,
            },
        );

        let client_messages = rx.await.unwrap();
        tokio::spawn(handle_bot_messages(player_id, event_bus, client_messages));

        Box::new(RemoteBotHandle {
            sender: server_msg_snd,
        })
    }
}

async fn handle_bot_messages(
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    mut messages: Streaming<pb::PlayerRequestResponse>,
) {
    while let Some(message) = messages.message().await.unwrap() {
        let request_id = (player_id, message.request_id as u32);
        event_bus
            .lock()
            .unwrap()
            .resolve_request(request_id, Ok(message.content));
    }
}

struct RemoteBotHandle {
    sender: mpsc::UnboundedSender<Result<pb::PlayerRequest, Status>>,
}

impl PlayerHandle for RemoteBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        self.sender
            .send(Ok(pb::PlayerRequest {
                request_id: r.request_id as i32,
                content: r.content,
            }))
            .unwrap();
    }
}

async fn run_match(router: PlayerRouter, pool: ConnectionPool) {
    let conn = pool.get().await.unwrap();

    let opponent = db::bots::find_bot_by_name("simplebot", &conn).unwrap();
    let opponent_code_bundle = db::bots::active_code_bundle(opponent.id, &conn).unwrap();

    let log_file_name = "remote_match.log";

    let remote_bot_spec = RemoteBotSpec { router };

    let match_config = runner::MatchConfig {
        map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
        map_name: "hex".to_string(),
        log_path: PathBuf::from(MATCHES_DIR).join(&log_file_name),
        players: vec![
            runner::MatchPlayer {
                bot_spec: Box::new(remote_bot_spec),
            },
            runner::MatchPlayer {
                bot_spec: code_bundle_to_botspec(&opponent_code_bundle),
            },
        ],
    };

    runner::run_match(match_config).await;
}

pub async fn run_bot_api(pool: ConnectionPool) {
    let router = PlayerRouter::new();
    tokio::spawn(run_match(router.clone(), pool));
    let server = BotApiServer { router };

    let addr = SocketAddr::from(([127, 0, 0, 1], 50051));
    Server::builder()
        .add_service(pb::bot_api_service_server::BotApiServiceServer::new(server))
        .serve(addr)
        .await
        .unwrap()
}
