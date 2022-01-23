pub mod bot_runner;
pub mod docker_runner;
pub mod match_context;
pub mod pw_match;

use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use futures::{stream::FuturesOrdered, FutureExt, StreamExt};
use match_context::MatchCtx;
use planetwars_rules::PwConfig;
use serde::{Deserialize, Serialize};

use self::match_context::{EventBus, PlayerHandle};

pub struct MatchConfig {
    pub map_name: String,
    pub map_path: PathBuf,
    pub log_path: PathBuf,
    pub players: Vec<MatchPlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct MatchMeta {
    pub map_name: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerInfo {
    pub name: String,
}

pub struct MatchPlayer {
    pub name: String,
    pub bot_spec: Box<dyn BotSpec>,
}

#[async_trait]
pub trait BotSpec : Send + Sync{
    async fn run_bot(
        &self,
        player_id: u32,
        event_bus: Arc<Mutex<EventBus>>,
    ) -> Box<dyn PlayerHandle>;
}

pub async fn run_match(config: MatchConfig) {
    let pw_config = PwConfig {
        map_file: config.map_path,
        max_turns: 100,
    };

    let event_bus = Arc::new(Mutex::new(EventBus::new()));

    // start bots
    // TODO: what happens when a bot fails?
    let players = config
        .players
        .iter()
        .enumerate()
        .map(|(player_id, player)| {
            let player_id = (player_id + 1) as u32;
            start_bot(player_id, event_bus.clone(), &player.bot_spec)
        })
        .collect::<FuturesOrdered<_>>()
        // await all results
        .collect()
        .await;
    let mut log_file = std::fs::File::create(config.log_path).expect("could not create log file");

    // assemble the math meta struct
    let match_meta = MatchMeta {
        map_name: config.map_name.clone(),
        timestamp: chrono::Local::now(),
        players: config
            .players
            .iter()
            .map(|bot| PlayerInfo {
                name: bot.name.clone(),
            })
            .collect(),
    };
    write!(
        log_file,
        "{}\n",
        serde_json::to_string(&match_meta).unwrap()
    )
    .unwrap();

    let match_ctx = MatchCtx::new(event_bus, players, log_file);

    let match_state = pw_match::PwMatch::create(match_ctx, pw_config);
    match_state.run().await;
}

// writing this as a closure causes lifetime inference errors
async fn start_bot(
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    bot_spec: &Box<dyn BotSpec>,
) -> (u32, Box<dyn PlayerHandle>) {
    let player_handle = bot_spec.run_bot(player_id, event_bus).await;
    (player_id, player_handle)
}