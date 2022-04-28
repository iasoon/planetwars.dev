use crate::db;
use crate::db::matches::{FullMatchData, FullMatchPlayerData};
use crate::modules::bots::save_code_bundle;
use crate::modules::matches::RunMatch;
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
) -> Result<Json<SubmitBotResponse>, StatusCode> {
    let conn = pool.get().await.expect("could not get database connection");

    let opponent_name = params
        .opponent_name
        .unwrap_or_else(|| DEFAULT_OPPONENT_NAME.to_string());

    let opponent =
        db::bots::find_bot_by_name(&opponent_name, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;
    let opponent_code_bundle =
        db::bots::active_code_bundle(opponent.id, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;

    let player_code_bundle = save_code_bundle(&params.code, None, &conn)
        // TODO: can we recover from this?
        .expect("could not save bot code");

    let mut run_match = RunMatch::from_players(vec![&player_code_bundle, &opponent_code_bundle]);
    let match_data = run_match
        .store_in_database(&conn)
        .expect("failed to save match");
    run_match.spawn(pool.clone());

    // TODO: avoid clones
    let full_match_data = FullMatchData {
        base: match_data.base,
        match_players: vec![
            FullMatchPlayerData {
                base: match_data.match_players[0].clone(),
                code_bundle: player_code_bundle,
                bot: None,
            },
            FullMatchPlayerData {
                base: match_data.match_players[1].clone(),
                code_bundle: opponent_code_bundle,
                bot: Some(opponent),
            },
        ],
    };

    let api_match = super::matches::match_data_to_api(full_match_data);
    Ok(Json(SubmitBotResponse {
        match_data: api_match,
    }))
}
