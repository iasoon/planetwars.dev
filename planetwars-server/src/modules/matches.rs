use std::path::PathBuf;

use diesel::{PgConnection, QueryResult};
use planetwars_matchrunner::{self as runner, docker_runner::DockerBotSpec, BotSpec, MatchConfig};
use runner::MatchOutcome;
use tokio::task::JoinHandle;

use crate::{
    db::{self, matches::MatchData},
    util::gen_alphanumeric,
    ConnectionPool, BOTS_DIR, MAPS_DIR, MATCHES_DIR,
};

const PYTHON_IMAGE: &str = "python:3.10-slim-buster";

pub struct RunMatch<'a> {
    log_file_name: String,
    player_code_bundles: Vec<&'a db::bots::CodeBundle>,
    match_id: Option<i32>,
}

impl<'a> RunMatch<'a> {
    pub fn from_players(player_code_bundles: Vec<&'a db::bots::CodeBundle>) -> Self {
        let log_file_name = format!("{}.log", gen_alphanumeric(16));
        RunMatch {
            log_file_name,
            player_code_bundles,
            match_id: None,
        }
    }

    pub fn runner_config(&self) -> runner::MatchConfig {
        runner::MatchConfig {
            map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
            map_name: "hex".to_string(),
            log_path: PathBuf::from(MATCHES_DIR).join(&self.log_file_name),
            players: self
                .player_code_bundles
                .iter()
                .map(|b| runner::MatchPlayer {
                    bot_spec: code_bundle_to_botspec(b),
                })
                .collect(),
        }
    }

    pub fn store_in_database(&mut self, db_conn: &PgConnection) -> QueryResult<MatchData> {
        // don't store the same match twice
        assert!(self.match_id.is_none());

        let new_match_data = db::matches::NewMatch {
            state: db::matches::MatchState::Playing,
            log_path: &self.log_file_name,
        };
        let new_match_players = self
            .player_code_bundles
            .iter()
            .map(|b| db::matches::MatchPlayerData {
                code_bundle_id: b.id,
            })
            .collect::<Vec<_>>();

        let match_data = db::matches::create_match(&new_match_data, &new_match_players, &db_conn)?;
        self.match_id = Some(match_data.base.id);
        Ok(match_data)
    }

    pub fn spawn(self, pool: ConnectionPool) -> JoinHandle<MatchOutcome> {
        let match_id = self.match_id.expect("match must be saved before running");
        let runner_config = self.runner_config();
        tokio::spawn(run_match_task(pool, runner_config, match_id))
    }
}

pub fn code_bundle_to_botspec(code_bundle: &db::bots::CodeBundle) -> Box<dyn BotSpec> {
    let bundle_path = PathBuf::from(BOTS_DIR).join(&code_bundle.path);

    Box::new(DockerBotSpec {
        code_path: bundle_path,
        image: PYTHON_IMAGE.to_string(),
        argv: vec!["python".to_string(), "bot.py".to_string()],
    })
}

async fn run_match_task(
    connection_pool: ConnectionPool,
    match_config: MatchConfig,
    match_id: i32,
) -> MatchOutcome {
    let outcome = runner::run_match(match_config).await;

    // update match state in database
    let conn = connection_pool
        .get()
        .await
        .expect("could not get database connection");

    db::matches::set_match_state(match_id, db::matches::MatchState::Finished, &conn)
        .expect("could not update match state");

    return outcome;
}
