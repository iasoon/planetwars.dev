use std::sync::Arc;

use crate::db;
use crate::db::matches::{FullMatchData, FullMatchPlayerData};
use crate::modules::bots::save_code_string;
use crate::modules::matches::{MatchPlayer, RunMatch};
use crate::ConnectionPool;
use crate::GlobalConfig;
use axum::extract::Extension;
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use super::matches::ApiMatch;

const DEFAULT_OPPONENT_NAME: &str = "simplebot";
const DEFAULT_MAP_NAME: &str = "hex";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotParams {
    pub code: String,
    pub opponent_name: Option<String>,
    pub map_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitBotResponse {
    #[serde(rename = "match")]
    pub match_data: ApiMatch,
}

/// Submit bot code and opponent name to play a match
pub async fn submit_bot(
    Json(params): Json<SubmitBotParams>,
    Extension(pool): Extension<ConnectionPool>,
    Extension(config): Extension<Arc<GlobalConfig>>,
) -> Result<Json<SubmitBotResponse>, StatusCode> {
    let mut conn = pool.get().await.expect("could not get database connection");

    let opponent_name = params
        .opponent_name
        .unwrap_or_else(|| DEFAULT_OPPONENT_NAME.to_string());

    let map_name = params
        .map_name
        .unwrap_or_else(|| DEFAULT_MAP_NAME.to_string());

    let (opponent_bot, opponent_bot_version) =
        db::bots::find_bot_with_version_by_name(&opponent_name, &mut conn)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

    let map =
        db::maps::find_map_by_name(&map_name, &mut conn).map_err(|_| StatusCode::BAD_REQUEST)?;

    let player_bot_version = save_code_string(&params.code, None, &mut conn, &config)
        // TODO: can we recover from this?
        .expect("could not save bot code");

    let run_match = RunMatch::new(
        config,
        false,
        map.clone(),
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
        map: Some(map),
    };

    let api_match = super::matches::match_data_to_api(full_match_data);
    Ok(Json(SubmitBotResponse {
        match_data: api_match,
    }))
}
