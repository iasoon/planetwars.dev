use futures::task::{Context, Poll};
use futures::{future::Future, task::AtomicWaker};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::match_log::{MatchLogMessage, MatchLogger};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMessage {
    pub request_id: u32,
    pub timeout: Duration,
    pub content: Vec<u8>,
}

pub struct MatchCtx {
    event_bus: Arc<Mutex<EventBus>>,
    players: HashMap<u32, PlayerData>,
    match_logger: MatchLogger,
}

impl MatchCtx {
    pub fn new(
        event_bus: Arc<Mutex<EventBus>>,
        players: HashMap<u32, Box<dyn PlayerHandle>>,
        match_logger: MatchLogger,
    ) -> Self {
        MatchCtx {
            event_bus,
            players: players
                .into_iter()
                .map(|(id, handle)| {
                    let player_handle = PlayerData {
                        request_ctr: 0,
                        handle,
                    };
                    (id, player_handle)
                })
                .collect(),
            match_logger,
        }
    }

    // TODO: implement a clean way to handle the player not existing
    pub fn request(&mut self, player_id: u32, content: Vec<u8>, timeout: Duration) -> Request {
        let player = self.players.get_mut(&player_id).unwrap();
        let request_id = player.request_ctr;
        player.request_ctr += 1;

        player.handle.send_request(RequestMessage {
            request_id,
            content,
            timeout,
        });

        return Request {
            player_id,
            request_id,
            event_bus: self.event_bus.clone(),
        };
    }

    pub fn players(&self) -> Vec<u32> {
        self.players.keys().cloned().collect()
    }

    pub fn log(&mut self, message: MatchLogMessage) {
        self.match_logger.send(message).expect("write failed");
    }
}

pub trait PlayerHandle: Send {
    fn send_request(&mut self, r: RequestMessage);
}

struct PlayerData {
    request_ctr: u32,
    handle: Box<dyn PlayerHandle>,
}

type RequestId = (u32, u32);
pub struct EventBus {
    request_responses: HashMap<RequestId, RequestResult<Vec<u8>>>,
    wakers: HashMap<RequestId, AtomicWaker>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            request_responses: HashMap::new(),
            wakers: HashMap::new(),
        }
    }
}

impl EventBus {
    pub fn resolve_request(&mut self, id: RequestId, result: RequestResult<Vec<u8>>) {
        if self.request_responses.contains_key(&id) {
            // request already resolved
            // TODO: maybe report this?
            return;
        }
        self.request_responses.insert(id, result);
        if let Some(waker) = self.wakers.remove(&id) {
            waker.wake();
        }
    }
}

pub struct Request {
    player_id: u32,
    request_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
}

impl Request {
    #[allow(dead_code)]
    pub fn player_id(&self) -> u32 {
        self.player_id
    }
}

impl Future for Request {
    type Output = RequestResult<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut event_bus = self.event_bus.lock().unwrap();
        let request_id = (self.player_id, self.request_id);

        if let Some(result) = event_bus.request_responses.get(&request_id) {
            return Poll::Ready(result.clone());
        }

        event_bus
            .wakers
            .entry(request_id)
            .or_insert_with(|| AtomicWaker::new())
            .register(cx.waker());
        return Poll::Pending;
    }
}

#[derive(Debug, Clone)]
pub enum RequestError {
    Timeout,
}

pub type RequestResult<T> = Result<T, RequestError>;
