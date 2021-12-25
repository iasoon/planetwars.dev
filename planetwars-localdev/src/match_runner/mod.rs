mod bot_runner;
mod match_context;
mod pw_match;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use match_context::MatchCtx;
use planetwars_rules::PwConfig;

use crate::BotConfig;

use self::match_context::{EventBus, PlayerHandle};

pub struct MatchConfig {
    pub map_path: PathBuf,
    pub log_path: PathBuf,
    pub players: Vec<MatchBot>,
}

pub struct MatchBot {
    pub name: String,
    pub bot_config: BotConfig,
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
        .map(|(player_id, bot)| {
            let player_id = (player_id + 1) as u32;
            let bot = bot_runner::Bot {
                working_dir: bot.bot_config.path.clone(),
                argv: bot.bot_config.argv.clone(),
            };
            let handle = bot_runner::run_local_bot(player_id, event_bus.clone(), bot);
            (player_id, Box::new(handle) as Box<dyn PlayerHandle>)
        })
        .collect();
    let log_file = std::fs::File::create(config.log_path).expect("could not create log file");
    let match_ctx = MatchCtx::new(event_bus, players, log_file);

    let match_state = pw_match::PwMatch::create(match_ctx, pw_config);
    match_state.run().await;
}
