use std::path::PathBuf;

use axum::Json;
use hyper::StatusCode;
use planetwars_matchrunner::{run_match, MatchConfig, MatchPlayer};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    db::{bots, users::User},
    DatabaseConnection, BOTS_DIR, MAPS_DIR, MATCHES_DIR,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct MatchParams {
    // Just bot ids for now
    players: Vec<i32>,
}

pub async fn play_match(
    user: User,
    conn: DatabaseConnection,
    Json(params): Json<MatchParams>,
) -> Result<(), StatusCode> {
    let map_path = PathBuf::from(MAPS_DIR).join("hex.json");

    let slug: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let log_path = PathBuf::from(MATCHES_DIR).join(&format!("{}.log", slug));

    let mut players = Vec::new();
    for bot_name in params.players {
        let bot = bots::find_bot(bot_name, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;
        let code_bundle =
            bots::active_code_bundle(bot.id, &conn).map_err(|_| StatusCode::BAD_REQUEST)?;

        let bundle_path = PathBuf::from(BOTS_DIR).join(&code_bundle.path);
        let bot_config: BotConfig = std::fs::read_to_string(bundle_path.join("botconfig.toml"))
            .and_then(|config_str| toml::from_str(&config_str).map_err(|e| e.into()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        players.push(MatchPlayer {
            name: bot.name.clone(),
            path: PathBuf::from(BOTS_DIR).join(code_bundle.path),
            argv: shlex::split(&bot_config.run_command)
                // TODO: this is an user error, should ideally be handled before we get here
                .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?,
        });
    }

    let match_config = MatchConfig {
        map_name: "hex".to_string(),
        map_path,
        log_path: log_path.clone(),
        players,
    };

    tokio::spawn(run_match(match_config));
    Ok(())
}

// TODO: this is duplicated from planetwars-cli
// clean this up and move to matchrunner crate
#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub run_command: String,
    pub build_command: Option<String>,
}
