use std::path::PathBuf;

use axum::{
    extract::{Extension, Path},
    Json,
};
use hyper::StatusCode;
use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        bots,
        matches::{self, MatchState},
        users::User,
    },
    ConnectionPool, DatabaseConnection, BOTS_DIR, MAPS_DIR, MATCHES_DIR,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct MatchParams {
    // Just bot ids for now
    players: Vec<i32>,
}

pub async fn play_match(
    _user: User,
    Extension(pool): Extension<ConnectionPool>,
    Json(params): Json<MatchParams>,
) -> Result<(), StatusCode> {
    let conn = pool.get().await.expect("could not get database connection");
    let map_path = PathBuf::from(MAPS_DIR).join("hex.json");

    let slug: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let log_file_name = format!("{}.log", slug);

    let mut players = Vec::new();
    let mut bot_ids = Vec::new();
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
            bot_spec: Box::new(DockerBotSpec {
                code_path: PathBuf::from(BOTS_DIR).join(code_bundle.path),
                image: "python:3.10-slim-buster".to_string(),
                argv: shlex::split(&bot_config.run_command)
                    // TODO: this is an user error, should ideally be handled before we get here
                    .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?,
            }),
        });

        bot_ids.push(matches::MatchPlayerData { bot_id: bot.id });
    }

    let match_config = MatchConfig {
        map_name: "hex".to_string(),
        map_path,
        log_path: PathBuf::from(MATCHES_DIR).join(&log_file_name),
        players,
    };

    tokio::spawn(run_match_task(
        match_config,
        log_file_name,
        bot_ids,
        pool.clone(),
    ));
    Ok(())
}

async fn run_match_task(
    config: MatchConfig,
    log_file_name: String,
    match_players: Vec<matches::MatchPlayerData>,
    pool: ConnectionPool,
) {
    let match_data = matches::NewMatch {
        state: MatchState::Finished,
        log_path: &log_file_name,
    };

    run_match(config).await;
    let conn = pool.get().await.expect("could not get database connection");
    matches::create_match(&match_data, &match_players, &conn).expect("could not create match");
}

#[derive(Serialize, Deserialize)]
pub struct ApiMatch {
    id: i32,
    timestamp: chrono::NaiveDateTime,
    players: Vec<ApiMatchPlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiMatchPlayer {
    bot_id: i32,
}

pub async fn list_matches(conn: DatabaseConnection) -> Result<Json<Vec<ApiMatch>>, StatusCode> {
    matches::list_matches(&conn)
        .map_err(|_| StatusCode::BAD_REQUEST)
        .map(|matches| Json(matches.into_iter().map(match_data_to_api).collect()))
}

fn match_data_to_api(data: matches::MatchData) -> ApiMatch {
    ApiMatch {
        id: data.base.id,
        timestamp: data.base.created_at,
        players: data
            .match_players
            .iter()
            .map(|p| ApiMatchPlayer { bot_id: p.bot_id })
            .collect(),
    }
}

// TODO: this is duplicated from planetwars-cli
// clean this up and move to matchrunner crate
#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub run_command: String,
    pub build_command: Option<String>,
}

pub async fn get_match_log(
    Path(match_id): Path<i32>,
    conn: DatabaseConnection,
) -> Result<Vec<u8>, StatusCode> {
    let match_base = matches::find_mach_base(match_id, &conn).map_err(|_| StatusCode::NOT_FOUND)?;
    let log_path = PathBuf::from(MATCHES_DIR).join(&match_base.log_path);
    let log_contents = std::fs::read(log_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(log_contents)
}
