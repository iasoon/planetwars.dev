pub mod pb {
    tonic::include_proto!("grpc.planetwars.bot_api");
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use runner::match_context::{EventBus, PlayerHandle, RequestError, RequestMessage};
use runner::match_log::MatchLogger;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};

use planetwars_matchrunner as runner;

use crate::db;
use crate::util::gen_alphanumeric;
use crate::ConnectionPool;
use crate::GlobalConfig;

use super::matches::{MatchPlayer, RunMatch};

pub struct BotApiServer {
    conn_pool: ConnectionPool,
    runner_config: Arc<GlobalConfig>,
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

impl Default for PlayerRouter {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: implement a way to expire entries
impl PlayerRouter {
    fn put(&self, player_key: String, entry: SyncThingData) {
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.insert(player_key, entry);
    }

    fn take(&self, player_key: &str) -> Option<SyncThingData> {
        // TODO: this design does not allow for reconnects. Is this desired?
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.remove(player_key)
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
        let player_key = req
            .metadata()
            .get("player_key")
            .ok_or_else(|| Status::unauthenticated("no player_key provided"))?;

        let player_key_str = player_key
            .to_str()
            .map_err(|_| Status::invalid_argument("unreadable string"))?;

        let sync_data = self
            .router
            .take(player_key_str)
            .ok_or_else(|| Status::not_found("player_key not found"))?;

        let stream = req.into_inner();

        sync_data.tx.send(stream).unwrap();
        Ok(Response::new(UnboundedReceiverStream::new(
            sync_data.server_messages,
        )))
    }

    async fn create_match(
        &self,
        req: Request<pb::MatchRequest>,
    ) -> Result<Response<pb::CreatedMatch>, Status> {
        // TODO: unify with matchrunner module
        let conn = self.conn_pool.get().await.unwrap();

        let match_request = req.get_ref();

        let opponent_bot = db::bots::find_bot_by_name(&match_request.opponent_name, &conn)
            .map_err(|_| Status::not_found("opponent not found"))?;
        let opponent_bot_version = db::bots::active_bot_version(opponent_bot.id, &conn)
            .map_err(|_| Status::not_found("no opponent version found"))?;

        let player_key = gen_alphanumeric(32);

        let remote_bot_spec = Box::new(RemoteBotSpec {
            player_key: player_key.clone(),
            router: self.router.clone(),
        });
        let run_match = RunMatch::from_players(
            self.runner_config.clone(),
            vec![
                MatchPlayer::BotSpec {
                    spec: remote_bot_spec,
                },
                MatchPlayer::BotVersion {
                    bot: Some(opponent_bot),
                    version: opponent_bot_version,
                },
            ],
        );
        let (created_match, _) = run_match
            .run(self.conn_pool.clone())
            .await
            .expect("failed to create match");

        Ok(Response::new(pb::CreatedMatch {
            match_id: created_match.base.id,
            player_key,
        }))
    }
}

// TODO: please rename me
struct SyncThingData {
    tx: oneshot::Sender<Streaming<pb::PlayerRequestResponse>>,
    server_messages: mpsc::UnboundedReceiver<Result<pb::PlayerRequest, Status>>,
}

struct RemoteBotSpec {
    player_key: String,
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
            self.player_key.clone(),
            SyncThingData {
                tx,
                server_messages: server_msg_recv,
            },
        );

        let fut = tokio::time::timeout(Duration::from_secs(10), rx);
        match fut.await {
            Ok(Ok(client_messages)) => {
                // let client_messages = rx.await.unwrap();
                tokio::spawn(handle_bot_messages(
                    player_id,
                    event_bus.clone(),
                    client_messages,
                ));
            }
            _ => {
                // ensure router cleanup
                self.router.take(&self.player_key);
            }
        };

        // If the player did not connect, the receiving half of `sender`
        // will be dropped here, resulting in a time-out for every turn.
        // This is fine for now, but
        // TODO: provide a formal mechanism for player startup failure
        Box::new(RemoteBotHandle {
            sender: server_msg_snd,
            player_id,
            event_bus,
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
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
}

impl PlayerHandle for RemoteBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        let res = self.sender.send(Ok(pb::PlayerRequest {
            request_id: r.request_id as i32,
            content: r.content,
        }));
        match res {
            Ok(()) => {
                // schedule a timeout. See comments at method implementation
                tokio::spawn(schedule_timeout(
                    (self.player_id, r.request_id),
                    r.timeout,
                    self.event_bus.clone(),
                ));
            }
            Err(_send_error) => {
                // cannot contact the remote bot anymore;
                // directly mark all requests as timed out.
                // TODO: create a dedicated error type for this.
                // should it be logged?
                println!("send error: {:?}", _send_error);
                self.event_bus
                    .lock()
                    .unwrap()
                    .resolve_request((self.player_id, r.request_id), Err(RequestError::Timeout));
            }
        }
    }
}

// TODO: this will spawn a task for every request, which might not be ideal.
// Some alternatives:
//  - create a single task that manages all time-outs.
//  - intersperse timeouts with incoming client messages
//  - push timeouts upwards, into the matchrunner logic (before we hit the playerhandle).
//    This was initially not done to allow timer start to be delayed until the message actually arrived
//    with the player. Is this still needed, or is there a different way to do this?
//
async fn schedule_timeout(
    request_id: (u32, u32),
    duration: Duration,
    event_bus: Arc<Mutex<EventBus>>,
) {
    tokio::time::sleep(duration).await;
    event_bus
        .lock()
        .unwrap()
        .resolve_request(request_id, Err(RequestError::Timeout));
}

pub async fn run_bot_api(runner_config: Arc<GlobalConfig>, pool: ConnectionPool) {
    let router = PlayerRouter::new();
    let server = BotApiServer {
        router,
        conn_pool: pool,
        runner_config,
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 50051));
    Server::builder()
        .add_service(pb::bot_api_service_server::BotApiServiceServer::new(server))
        .serve(addr)
        .await
        .unwrap()
}
