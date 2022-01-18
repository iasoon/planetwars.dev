pub mod bot_runner;
pub mod match_context;
pub mod pw_match;

use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

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
    pub path: PathBuf,
    pub argv: Vec<String>,
}

pub async fn run_match(config: MatchConfig) {
    let pw_config = PwConfig {
        map_file: config.map_path,
        max_turns: 100,
    };

    let event_bus = Arc::new(Mutex::new(EventBus::new()));

    // start bots
    let players = config
        .players
        .iter()
        .enumerate()
        .map(|(player_id, player)| {
            let player_id = (player_id + 1) as u32;
            let bot = bot_runner::Bot {
                working_dir: player.path.clone(),
                argv: player.argv.clone(),
            };
            let handle = bot_runner::run_local_bot(player_id, event_bus.clone(), bot);
            (player_id, Box::new(handle) as Box<dyn PlayerHandle>)
        })
        .collect();
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
