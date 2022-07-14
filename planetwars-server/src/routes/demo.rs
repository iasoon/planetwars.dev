use std::sync::Arc;

use crate::db;
use crate::db::matches::{FullMatchData, FullMatchPlayerData};
use crate::modules::bots::save_code_string;
use crate::modules::matches::{MatchPlayer, MatchRunnerConfig, RunMatch};
use crate::ConnectionPool;
use axum::extract::Extension;
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use super::matches::ApiMatch;

const DEFAULT_OPPONENT_NAME: &str = "simplebot";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotParams {
    pub code: String,
    // TODO: would it be better to pass an ID here?
    pub opponent_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitBotResponse {
    #[serde(rename = "match")]
    pub match_data: ApiMatch,
}

/// submit python code for a bot, which will face off
/// with a demo bot. Return a played match.
pub async fn submit_bot(
    Json(params): Json<SubmitBotParams>,
    Extension(pool): Extension<ConnectionPool>,
    Extension(runner_config): Extension<Arc<MatchRunnerConfig>>,
) -> Result<Json<SubmitBotResponse>, StatusCode> {
    let conn = pool.get().await.expect("could not get database connection");

    let opponent_name = params
        .opponent_name
        .unwrap_or_else(|| DEFAULT_OPPONENT_NAME.to_string());

    let opponent_bot =
        db::bots::find_bot_by_name(&opponent_name, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;
    let opponent_bot_version = db::bots::active_bot_version(opponent_bot.id, &conn)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let player_bot_version = save_code_string(&params.code, None, &conn)
        // TODO: can we recover from this?
        .expect("could not save bot code");

    let run_match = RunMatch::from_players(
        runner_config,
        vec![
            MatchPlayer::BotVersion {
                bot: None,
                version: player_bot_version.clone(),
            },
            MatchPlayer::BotVersion {
                bot: Some(opponent_bot.clone()),
                version: opponent_bot_version.clone(),
            },
        ],
    );
    let (match_data, _) = run_match
        .run(pool.clone())
        .await
        .expect("failed to run match");

    // TODO: avoid clones
    let full_match_data = FullMatchData {
        base: match_data.base,
        match_players: vec![
            FullMatchPlayerData {
                base: match_data.match_players[0].clone(),
                bot_version: Some(player_bot_version),
                bot: None,
            },
            FullMatchPlayerData {
                base: match_data.match_players[1].clone(),
                bot_version: Some(opponent_bot_version),
                bot: Some(opponent_bot),
            },
        ],
    };

    let api_match = super::matches::match_data_to_api(full_match_data);
    Ok(Json(SubmitBotResponse {
        match_data: api_match,
    }))
}
