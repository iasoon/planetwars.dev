use std::path::PathBuf;

use axum::Json;
use hyper::StatusCode;
use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{DatabaseConnection, BOTS_DIR, MAPS_DIR, MATCHES_DIR};

const PYTHON_IMAGE: &'static str = "python:3.10-slim-buster";
const SIMPLEBOT_PATH: &'static str = "../simplebot";

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotParams {
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitBotResponse {
    pub match_id: String,
}

pub fn gen_alphanumeric(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// submit python code for a bot, which will face off
/// with a demo bot. Return a played match.
pub async fn submit_bot(
    Json(params): Json<SubmitBotParams>,
) -> Result<Json<SubmitBotResponse>, StatusCode> {
    let uploaded_bot_id: String = gen_alphanumeric(16);
    let match_id = gen_alphanumeric(16);

    let uploaded_bot_dir = PathBuf::from(BOTS_DIR).join(&uploaded_bot_id);
    std::fs::create_dir(&uploaded_bot_dir).unwrap();
    std::fs::write(uploaded_bot_dir.join("bot.py"), params.code.as_bytes()).unwrap();

    run_match(MatchConfig {
        map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
        map_name: "hex".to_string(),
        log_path: PathBuf::from(MATCHES_DIR).join(format!("{}.log", match_id)),
        players: vec![
            MatchPlayer {
                name: "bot".to_string(),
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

    Ok(Json(SubmitBotResponse { match_id }))
}
