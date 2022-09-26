use crate::match_context::RequestError;
use crate::match_log::MatchLogMessage;

use super::match_context::{MatchCtx, RequestResult};
use futures::stream::futures_unordered::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use serde_json;

use std::collections::HashMap;
use std::convert::TryInto;

pub use planetwars_rules::config::{Config, Map};

use planetwars_rules::protocol as proto;
use planetwars_rules::serializer as pw_serializer;
use planetwars_rules::{PlanetWars, PwConfig};

pub struct PwMatch {
    pub match_ctx: MatchCtx,
    pub match_state: PlanetWars,
    pub player_status: HashMap<usize, PlayerStatus>,
}

pub struct PlayerStatus {
    pub had_command_errors: bool,
    pub terminated: bool,
}

impl PwMatch {
    pub fn create(match_ctx: MatchCtx, config: PwConfig) -> Self {
        // TODO: this is kind of hacked together at the moment
        let match_state = PlanetWars::create(config, match_ctx.players().len());
        let player_status = match_ctx
            .players()
            .into_iter()
            .map(|player_id| {
                (
                    player_id as usize,
                    PlayerStatus {
                        had_command_errors: false,
                        terminated: false,
                    },
                )
            })
            .collect();

        PwMatch {
            match_state,
            match_ctx,
            player_status,
        }
    }

    pub async fn run(&mut self) {
        // log initial state
        self.log_game_state();

        while !self.match_state.is_finished() {
            let player_messages = self.prompt_players().await;

            for (player_id, turn) in player_messages {
                let player_action = self.execute_action(player_id, turn);
                self.update_player_status(player_id, &player_action);
                self.log_player_action(player_id, player_action);
            }
            self.match_state.step();
            self.log_game_state();
        }
    }

    async fn prompt_players(&mut self) -> Vec<(usize, RequestResult<Vec<u8>>)> {
        // borrow these outside closure to make the borrow checker happy
        let state = self.match_state.state();
        let match_ctx = &mut self.match_ctx;

        // TODO: this numbering is really messy.
        // Get rid of the distinction between player_num
        // and player_id.

        self.match_state
            .state()
            .players
            .iter()
            .filter(|p| p.alive)
            .map(move |player| {
                let state_for_player = pw_serializer::serialize_rotated(state, player.id - 1);
                match_ctx
                    .request(
                        player.id.try_into().unwrap(),
                        serde_json::to_vec(&state_for_player).unwrap(),
                        Duration::from_millis(1000),
                    )
                    .map(move |resp| (player.id, resp))
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
    }

    fn execute_action(&mut self, player_num: usize, turn: RequestResult<Vec<u8>>) -> PlayerAction {
        let data = match turn {
            Err(RequestError::Timeout) => return PlayerAction::Timeout,
            Err(RequestError::BotTerminated) => return PlayerAction::Terminated,
            Ok(data) => data,
        };

        let action: proto::Action = match serde_json::from_slice(&data) {
            Err(error) => return PlayerAction::ParseError { data, error },
            Ok(action) => action,
        };

        let commands = action
            .commands
            .into_iter()
            .map(|command| {
                let res = self.match_state.execute_command(player_num, &command);
                PlayerCommand {
                    command,
                    error: res.err(),
                }
            })
            .collect();

        PlayerAction::Commands(commands)
    }

    fn log_game_state(&mut self) {
        let state = self.match_state.serialize_state();
        self.match_ctx.log(MatchLogMessage::GameState(state));
    }

    fn log_player_action(&mut self, player_id: usize, player_action: PlayerAction) {
        match player_action {
            PlayerAction::Timeout => self.match_ctx.log(MatchLogMessage::Timeout {
                player_id: player_id as u32,
            }),
            PlayerAction::Terminated => (), // TODO: should something be logged here?
            PlayerAction::ParseError { data, error } => {
                // TODO: can this be handled better?
                let command =
                    String::from_utf8(data).unwrap_or_else(|_| "<invalid utf-8>".to_string());

                self.match_ctx.log(MatchLogMessage::BadCommand {
                    player_id: player_id as u32,
                    command,
                    error: error.to_string(),
                });
            }
            PlayerAction::Commands(dispatches) => {
                self.match_ctx.log(MatchLogMessage::Dispatches {
                    player_id: player_id as u32,
                    dispatches,
                });
            }
        }
    }

    fn update_player_status(&mut self, player_id: usize, player_action: &PlayerAction) {
        let player_status = self.player_status.get_mut(&player_id).unwrap();
        match player_action {
            PlayerAction::Commands(dispatches) => {
                if dispatches.iter().any(|d| d.error.is_some()) {
                    player_status.had_command_errors = true;
                }
            }
            PlayerAction::ParseError { .. } => {
                player_status.had_command_errors = true;
            }
            PlayerAction::Timeout => {
                player_status.had_command_errors = true;
            }
            PlayerAction::Terminated => {
                player_status.terminated = true;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCommand {
    #[serde(flatten)]
    pub command: proto::Command,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<proto::CommandError>,
}

/// the action a player performed.
// TODO: can we name this better? Is this a "play"?
pub enum PlayerAction {
    Timeout,
    Terminated,
    ParseError {
        data: Vec<u8>,
        error: serde_json::Error,
    },
    Commands(Vec<PlayerCommand>),
}
