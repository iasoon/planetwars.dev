pub mod bot_runner;
pub mod docker_runner;
pub mod match_context;
pub mod match_log;
pub mod pw_match;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use futures::{stream::FuturesOrdered, StreamExt};
use match_context::MatchCtx;
use match_log::{create_log_sink, MatchLogger};
use planetwars_rules::PwConfig;

pub use self::match_context::{EventBus, PlayerHandle};

pub struct MatchConfig {
    pub map_name: String,
    pub map_path: PathBuf,
    pub log_path: PathBuf,
    pub players: Vec<MatchPlayer>,
}

pub struct MatchPlayer {
    pub bot_spec: Box<dyn BotSpec>,
}

#[async_trait]
pub trait BotSpec: Send + Sync {
    async fn run_bot(
        &self,
        player_id: u32,
        event_bus: Arc<Mutex<EventBus>>,
        match_logger: MatchLogger,
    ) -> Box<dyn PlayerHandle>;
}

pub struct MatchOutcome {
    pub winner: Option<usize>,
    pub player_outcomes: Vec<PlayerOutcome>,
}

pub struct PlayerOutcome {
    pub had_errors: bool,
    pub crashed: bool,
}

pub async fn run_match(config: MatchConfig) -> MatchOutcome {
    let pw_config = PwConfig {
        map_file: config.map_path,
        max_turns: 500,
    };

    let event_bus = Arc::new(Mutex::new(EventBus::new()));
    let match_logger = create_log_sink(&config.log_path).await;

    // start bots
    // TODO: what happens when a bot fails?
    let players = config
        .players
        .iter()
        .enumerate()
        .map(|(player_id, player)| {
            let player_id = (player_id + 1) as u32;
            start_bot(
                player_id,
                event_bus.clone(),
                player.bot_spec.as_ref(),
                match_logger.clone(),
            )
        })
        .collect::<FuturesOrdered<_>>()
        // await all results
        .collect()
        .await;

    let match_ctx = MatchCtx::new(event_bus, players, match_logger);

    let mut match_instance = pw_match::PwMatch::create(match_ctx, pw_config);
    match_instance.run().await;
    match_instance.match_ctx.shutdown().await;

    let survivors = match_instance.match_state.state().living_players();
    let winner = if survivors.len() == 1 {
        Some(survivors[0])
    } else {
        None
    };

    let player_outcomes = (1..=config.players.len())
        .map(|player_id| {
            let player_status = &match_instance.player_status[&player_id];
            PlayerOutcome {
                had_errors: player_status.had_command_errors,
                crashed: player_status.terminated,
            }
        })
        .collect();

    MatchOutcome {
        winner,
        player_outcomes,
    }
}

// writing this as a closure causes lifetime inference errors
async fn start_bot(
    player_id: u32,
    event_bus: Arc<Mutex<EventBus>>,
    bot_spec: &dyn BotSpec,
    match_logger: MatchLogger,
) -> (u32, Box<dyn PlayerHandle>) {
    let player_handle = bot_spec.run_bot(player_id, event_bus, match_logger).await;
    (player_id, player_handle)
}
