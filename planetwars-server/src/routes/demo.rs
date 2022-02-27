use crate::db;
use crate::db::matches::MatchState;
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

const PYTHON_IMAGE: &'static str = "python:3.10-slim-buster";
const OPPONENT_NAME: &'static str = "simplebot";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotParams {
    pub bot_name: Option<String>,
    pub code: String,
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

    let opponent =
        db::bots::find_bot_by_name(OPPONENT_NAME, &conn).expect("could not find opponent bot");
    let opponent_code_bundle =
        db::bots::active_code_bundle(opponent.id, &conn).expect("opponent bot has no code bundles");

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
    // TODO: set match players
    let match_data =
        db::matches::create_match(&new_match_data, &[], &conn).expect("failed to create match");

    tokio::spawn(run_match_task(
        match_data.base.id,
        match_config,
        pool.clone(),
    ));

    let api_match = super::matches::match_data_to_api(match_data);
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
