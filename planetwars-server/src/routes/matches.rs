use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    db::matches::{self, MatchState},
    DatabaseConnection, MATCHES_DIR,
};

#[derive(Serialize, Deserialize)]
pub struct ApiMatch {
    id: i32,
    timestamp: chrono::NaiveDateTime,
    state: MatchState,
    players: Vec<ApiMatchPlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiMatchPlayer {
    code_bundle_id: Option<i32>,
    bot_id: Option<i32>,
    bot_name: Option<String>,
}

pub async fn list_matches(conn: DatabaseConnection) -> Result<Json<Vec<ApiMatch>>, StatusCode> {
    matches::list_matches(&conn)
        .map_err(|_| StatusCode::BAD_REQUEST)
        .map(|matches| Json(matches.into_iter().map(match_data_to_api).collect()))
}

pub fn match_data_to_api(data: matches::FullMatchData) -> ApiMatch {
    ApiMatch {
        id: data.base.id,
        timestamp: data.base.created_at,
        state: data.base.state,
        players: data
            .match_players
            .iter()
            .map(|_p| ApiMatchPlayer {
                code_bundle_id: _p.code_bundle.as_ref().map(|cb| cb.id),
                bot_id: _p.bot.as_ref().map(|b| b.id),
                bot_name: _p.bot.as_ref().map(|b| b.name.clone()),
            })
            .collect(),
    }
}

pub async fn get_match_data(
    Path(match_id): Path<i32>,
    conn: DatabaseConnection,
) -> Result<Json<ApiMatch>, StatusCode> {
    let match_data = matches::find_match(match_id, &conn)
        .map_err(|_| StatusCode::NOT_FOUND)
        .map(match_data_to_api)?;
    Ok(Json(match_data))
}

pub async fn get_match_log(
    Path(match_id): Path<i32>,
    conn: DatabaseConnection,
) -> Result<Vec<u8>, StatusCode> {
    let match_base =
        matches::find_match_base(match_id, &conn).map_err(|_| StatusCode::NOT_FOUND)?;
    let log_path = PathBuf::from(MATCHES_DIR).join(&match_base.log_path);
    let log_contents = std::fs::read(log_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(log_contents)
}
