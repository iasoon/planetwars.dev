use crate::db::matches::{self, MatchState};
use crate::{ConnectionPool, BOTS_DIR, MAPS_DIR, MATCHES_DIR};
use axum::extract::Extension;
use axum::Json;
use hyper::StatusCode;
use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::matches::ApiMatch;

const PYTHON_IMAGE: &'static str = "python:3.10-slim-buster";
const SIMPLEBOT_PATH: &'static str = "../simplebot";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotParams {
    pub code: String,
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

    let uploaded_bot_uuid: String = gen_alphanumeric(16);
    let log_file_name = format!("{}.log", gen_alphanumeric(16));

    // store uploaded bot
    let uploaded_bot_dir = PathBuf::from(BOTS_DIR).join(&uploaded_bot_uuid);
    std::fs::create_dir(&uploaded_bot_dir).unwrap();
    std::fs::write(uploaded_bot_dir.join("bot.py"), params.code.as_bytes()).unwrap();

    // play the match
    run_match(MatchConfig {
        map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
        map_name: "hex".to_string(),
        log_path: PathBuf::from(MATCHES_DIR).join(&log_file_name),
        players: vec![
            MatchPlayer {
                name: "player".to_string(),
                bot_spec: Box::new(DockerBotSpec {
                    code_path: uploaded_bot_dir,
                    image: PYTHON_IMAGE.to_string(),
                    argv: vec!["python".to_string(), "bot.py".to_string()],
                }),
            },
            MatchPlayer {
                name: "simplebot".to_string(),
                bot_spec: Box::new(DockerBotSpec {
                    code_path: PathBuf::from(SIMPLEBOT_PATH),
                    image: PYTHON_IMAGE.to_string(),
                    argv: vec!["python".to_string(), "simplebot.py".to_string()],
                }),
            },
        ],
    })
    .await;

    // store match in database
    let new_match_data = matches::NewMatch {
        state: MatchState::Finished,
        log_path: &log_file_name,
    };
    // TODO: set match players
    let match_data =
        matches::create_match(&new_match_data, &[], &conn).expect("failed to create match");

    let api_match = super::matches::match_data_to_api(match_data);
    Ok(Json(SubmitBotResponse {
        match_data: api_match,
    }))
}

pub fn gen_alphanumeric(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
