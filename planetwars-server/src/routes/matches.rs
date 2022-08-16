use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

use crate::{
    db::{
        self,
        matches::{self, MatchState},
    },
    DatabaseConnection, GlobalConfig,
};

#[derive(Serialize, Deserialize)]
pub struct ApiMatch {
    id: i32,
    timestamp: chrono::NaiveDateTime,
    state: MatchState,
    players: Vec<ApiMatchPlayer>,
    winner: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiMatchPlayer {
    bot_version_id: Option<i32>,
    bot_id: Option<i32>,
    bot_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRecentMatchesParams {
    count: Option<usize>,
    // TODO: should timezone be specified here?
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,

    bot: Option<String>,
}

const MAX_NUM_RETURNED_MATCHES: usize = 100;
const DEFAULT_NUM_RETURNED_MATCHES: usize = 50;

#[derive(Serialize, Deserialize)]
pub struct ListMatchesResponse {
    matches: Vec<ApiMatch>,
    has_next: bool,
}

pub async fn list_recent_matches(
    Query(params): Query<ListRecentMatchesParams>,
    conn: DatabaseConnection,
) -> Result<Json<ListMatchesResponse>, StatusCode> {
    let requested_count = std::cmp::min(
        params.count.unwrap_or(DEFAULT_NUM_RETURNED_MATCHES),
        MAX_NUM_RETURNED_MATCHES,
    );

    // fetch one additional record to check whether a next page exists
    let count = (requested_count + 1) as i64;

    let matches_result = match params.bot {
        Some(bot_name) => {
            let bot = db::bots::find_bot_by_name(&bot_name, &conn)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            matches::list_bot_matches(bot.id, count, params.before, params.after, &conn)
        }
        None => matches::list_public_matches(count, params.before, params.after, &conn),
    };

    let mut matches = matches_result.map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut has_next = false;
    if matches.len() > requested_count {
        has_next = true;
        matches.truncate(requested_count);
    }

    let api_matches = matches
        .into_iter()
        .map(match_data_to_api)
        .collect();
    
    Ok(Json(ListMatchesResponse {
        matches: api_matches,
        has_next,
    }))
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
                bot_version_id: _p.bot_version.as_ref().map(|cb| cb.id),
                bot_id: _p.bot.as_ref().map(|b| b.id),
                bot_name: _p.bot.as_ref().map(|b| b.name.clone()),
            })
            .collect(),
        winner: data.base.winner,
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
    Extension(config): Extension<Arc<GlobalConfig>>,
) -> Result<Vec<u8>, StatusCode> {
    let match_base =
        matches::find_match_base(match_id, &conn).map_err(|_| StatusCode::NOT_FOUND)?;
    let log_path = PathBuf::from(&config.match_logs_directory).join(&match_base.log_path);
    let log_contents = std::fs::read(log_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(log_contents)
}
