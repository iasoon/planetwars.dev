use crate::db;
use crate::db::matches::{FullMatchData, FullMatchPlayerData, MatchPlayerData, MatchState};
use crate::modules::bots::save_code_bundle;
use crate::util::gen_alphanumeric;
use crate::{ConnectionPool, BOTS_DIR, MAPS_DIR, MATCHES_DIR};
use axum::extract::Extension;
use axum::Json;
use hyper::StatusCode;
use planetwars_matchrunner::BotSpec;
use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::matches::ApiMatch;

const PYTHON_IMAGE: &str = "python:3.10-slim-buster";
const OPPONENT_NAME: &str = "simplebot";

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

fn code_bundle_to_botspec(code_bundle: &db::bots::CodeBundle) -> Box<dyn BotSpec> {
    let bundle_path = PathBuf::from(BOTS_DIR).join(&code_bundle.path);

    Box::new(DockerBotSpec {
        code_path: bundle_path,
        image: PYTHON_IMAGE.to_string(),
        argv: vec!["python".to_string(), "bot.py".to_string()],
    })
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
        .unwrap_or_else(|| OPPONENT_NAME.to_string());

    let opponent =
        db::bots::find_bot_by_name(&opponent_name, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;
    let opponent_code_bundle =
        db::bots::active_code_bundle(opponent.id, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;

    let player_code_bundle = save_code_bundle(&params.code, None, &conn)
        // TODO: can we recover from this?
        .expect("could not save bot code");

    let log_file_name = format!("{}.log", gen_alphanumeric(16));
    // play the match
    let match_config = MatchConfig {
        map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
        map_name: "hex".to_string(),
        log_path: PathBuf::from(MATCHES_DIR).join(&log_file_name),
        players: vec![
            MatchPlayer {
                name: "player".to_string(),
                bot_spec: code_bundle_to_botspec(&player_code_bundle),
            },
            MatchPlayer {
                name: OPPONENT_NAME.to_string(),
                bot_spec: code_bundle_to_botspec(&opponent_code_bundle),
            },
        ],
    };

    // store match in database
    let new_match_data = db::matches::NewMatch {
        state: MatchState::Playing,
        log_path: &log_file_name,
    };

    let new_match_players = [
        MatchPlayerData {
            code_bundle_id: player_code_bundle.id,
        },
        MatchPlayerData {
            code_bundle_id: opponent_code_bundle.id,
        },
    ];
    let match_data = db::matches::create_match(&new_match_data, &new_match_players, &conn)
        .expect("failed to create match");

    tokio::spawn(run_match_task(
        match_data.base.id,
        match_config,
        pool.clone(),
    ));

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

async fn run_match_task(match_id: i32, match_config: MatchConfig, connection_pool: ConnectionPool) {
    run_match(match_config).await;
    let conn = connection_pool
        .get()
        .await
        .expect("could not get database connection");
    db::matches::set_match_state(match_id, MatchState::Finished, &conn)
        .expect("failed to update match state");
}
