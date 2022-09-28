pub mod pb {
    tonic::include_proto!("grpc.planetwars.client_api");

    pub use player_api_client_message::ClientMessage as PlayerApiClientMessageType;
    pub use player_api_server_message::ServerMessage as PlayerApiServerMessageType;
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use runner::match_context::{EventBus, PlayerHandle, RequestError, RequestMessage};
use runner::match_log::MatchLogger;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
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

pub struct ClientApiServer {
    conn_pool: ConnectionPool,
    runner_config: Arc<GlobalConfig>,
    router: PlayerRouter,
}

type ClientMessages = Streaming<pb::PlayerApiClientMessage>;
type ServerMessages = mpsc::UnboundedReceiver<Result<pb::PlayerApiServerMessage, Status>>;

enum PlayerConnectionState {
    Reserved,
    ClientConnected {
        tx: oneshot::Sender<ServerMessages>,
        client_messages: ClientMessages,
    },
    ServerConnected {
        tx: oneshot::Sender<ClientMessages>,
        server_messages: ServerMessages,
    },
    // In connected state, the connection is removed from the PlayerRouter
}

/// Routes players to their handler
#[derive(Clone)]
struct PlayerRouter {
    routing_table: Arc<Mutex<HashMap<String, PlayerConnectionState>>>,
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
    fn put(&self, player_key: String, entry: PlayerConnectionState) {
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.insert(player_key, entry);
    }

    fn take(&self, player_key: &str) -> Option<PlayerConnectionState> {
        // TODO: this design does not allow for reconnects. Is this desired?
        let mut routing_table = self.routing_table.lock().unwrap();
        routing_table.remove(player_key)
    }
}

#[tonic::async_trait]
impl pb::client_api_service_server::ClientApiService for ClientApiServer {
    type ConnectPlayerStream = UnboundedReceiverStream<Result<pb::PlayerApiServerMessage, Status>>;

    async fn connect_player(
        &self,
        req: Request<Streaming<pb::PlayerApiClientMessage>>,
    ) -> Result<Response<Self::ConnectPlayerStream>, Status> {
        // TODO: clean up errors
        let player_key = req
            .metadata()
            .get("player_key")
            .ok_or_else(|| Status::unauthenticated("no player_key provided"))?;

        let player_key_string = player_key
            .to_str()
            .map_err(|_| Status::invalid_argument("unreadable string"))?
            .to_string();

        let client_messages = req.into_inner();

        let server_messages_promise = {
            // during this block, a lack is held on the routing table

            let mut routing_table = self.router.routing_table.lock().unwrap();
            let connection_state = routing_table
                .remove(&player_key_string)
                .ok_or_else(|| Status::not_found("player_key not found"))?;
            match connection_state {
                PlayerConnectionState::Reserved => {
                    let (tx, rx) = oneshot::channel();

                    routing_table.insert(
                        player_key_string,
                        PlayerConnectionState::ClientConnected {
                            tx,
                            client_messages,
                        },
                    );

                    Promise::Awaiting(rx)
                }
                PlayerConnectionState::ServerConnected {
                    tx,
                    server_messages,
                } => {
                    tx.send(client_messages).unwrap();
                    Promise::Resolved(server_messages)
                }
                PlayerConnectionState::ClientConnected { .. } => panic!("player already connected"),
            }
        };

        let server_messages = server_messages_promise
            .get_value()
            .await
            .map_err(|_| Status::internal("failed to connect player to game"))?;
        Ok(Response::new(UnboundedReceiverStream::new(server_messages)))
    }

    async fn create_match(
        &self,
        req: Request<pb::CreateMatchRequest>,
    ) -> Result<Response<pb::CreateMatchResponse>, Status> {
        // TODO: unify with matchrunner module
        let conn = self.conn_pool.get().await.unwrap();

        let match_request = req.get_ref();

        let (opponent_bot, opponent_bot_version) =
            db::bots::find_bot_with_version_by_name(&match_request.opponent_name, &conn)
                .map_err(|_| Status::not_found("opponent not found"))?;

        let map_name = match match_request.map_name.as_str() {
            "" => "hex",
            name => name,
        };
        let map = db::maps::find_map_by_name(map_name, &conn)
            .map_err(|_| Status::not_found("map not found"))?;

        let player_key = gen_alphanumeric(32);
        // ensure that the player key is registered in the router when we send a response
        self.router
            .put(player_key.clone(), PlayerConnectionState::Reserved);

        let remote_bot_spec = Box::new(RemoteBotSpec {
            player_key: player_key.clone(),
            router: self.router.clone(),
        });
        let run_match = RunMatch::new(
            self.runner_config.clone(),
            false,
            map,
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

        Ok(Response::new(pb::CreateMatchResponse {
            match_id: created_match.base.id,
            player_key,
            // TODO: can we avoid hardcoding this?
            match_url: format!(
                "{}/matches/{}",
                self.runner_config.root_url, created_match.base.id
            ),
        }))
    }
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
        let (server_msg_snd, server_msg_recv) = mpsc::unbounded_channel();

        let client_messages_promise = {
            // during this block, we hold a lock on the routing table.

            let mut routing_table = self.router.routing_table.lock().unwrap();
            let connection_state = routing_table
                .remove(&self.player_key)
                .expect("player key not found in routing table");

            match connection_state {
                PlayerConnectionState::Reserved => {
                    let (tx, rx) = oneshot::channel();
                    routing_table.insert(
                        self.player_key.clone(),
                        PlayerConnectionState::ServerConnected {
                            tx,
                            server_messages: server_msg_recv,
                        },
                    );
                    Promise::Awaiting(rx)
                }
                PlayerConnectionState::ClientConnected {
                    tx,
                    client_messages,
                } => {
                    tx.send(server_msg_recv).unwrap();
                    Promise::Resolved(client_messages)
                }
                PlayerConnectionState::ServerConnected { .. } => panic!("server already connected"),
            }
        };

        let client_messages_future =
            tokio::time::timeout(Duration::from_secs(10), client_messages_promise.get_value());

        let join_handle = if let Ok(Ok(client_messages)) = client_messages_future.await {
            tokio::spawn(handle_bot_messages(
                player_id,
                event_bus.clone(),
                client_messages,
            ))
        } else {
            // since we don't have a mechanism for bot failure yet,
            // this hack will have to do for now.
            // TODO: FIXME
            tokio::spawn(futures::future::ready(()))
        };

        // ensure router cleanup
        self.router.take(&self.player_key);

        // If the player did not connect, the receiving half of `sender`
        // will be dropped here, resulting in a time-out for every turn.
        // This is fine for now, but
        // TODO: provide a formal mechanism for player startup failure
        Box::new(RemoteBotHandle {
            sender: server_msg_snd,
            player_id,
            event_bus,
            join_handle,
        })
    }
}

async fn handle_bot_messages(
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    mut messages: Streaming<pb::PlayerApiClientMessage>,
) {
    // TODO: can this be written more nicely?
    while let Some(message) = messages.message().await.unwrap() {
        match message.client_message {
            Some(pb::PlayerApiClientMessageType::Action(resp)) => {
                let request_id = (player_id, resp.action_request_id as u32);
                event_bus
                    .lock()
                    .unwrap()
                    .resolve_request(request_id, Ok(resp.content));
            }
            _ => (),
        }
    }
}

struct RemoteBotHandle {
    sender: mpsc::UnboundedSender<Result<pb::PlayerApiServerMessage, Status>>,
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    join_handle: JoinHandle<()>,
}

impl PlayerHandle for RemoteBotHandle {
    fn send_request(&mut self, r: RequestMessage) {
        let req = pb::PlayerActionRequest {
            action_request_id: r.request_id as i32,
            content: r.content,
        };

        let server_message = pb::PlayerApiServerMessage {
            server_message: Some(pb::PlayerApiServerMessageType::ActionRequest(req)),
        };

        let res = self.sender.send(Ok(server_message));
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

    fn into_join_handle(self: Box<Self>) -> JoinHandle<()> {
        self.join_handle
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

pub async fn run_client_api(runner_config: Arc<GlobalConfig>, pool: ConnectionPool) {
    let router = PlayerRouter::new();
    let server = ClientApiServer {
        router,
        conn_pool: pool,
        runner_config,
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 50051));
    Server::builder()
        .add_service(pb::client_api_service_server::ClientApiServiceServer::new(
            server,
        ))
        .serve(addr)
        .await
        .unwrap()
}

enum Promise<T> {
    Resolved(T),
    Awaiting(oneshot::Receiver<T>),
}

impl<T> Promise<T> {
    async fn get_value(self) -> Result<T, oneshot::error::RecvError> {
        match self {
            Promise::Resolved(val) => Ok(val),
            Promise::Awaiting(rx) => rx.await,
        }
    }
}
