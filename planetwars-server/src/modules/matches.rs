use std::{path::PathBuf, sync::Arc};

use diesel::{PgConnection, QueryResult};
use planetwars_matchrunner::{self as runner, docker_runner::DockerBotSpec, BotSpec, MatchConfig};
use runner::MatchOutcome;
use tokio::task::JoinHandle;

use crate::{
    db::{
        self,
        matches::{MatchData, MatchResult},
    },
    util::gen_alphanumeric,
    ConnectionPool, GlobalConfig, BOTS_DIR, MAPS_DIR, MATCHES_DIR,
};

pub struct RunMatch {
    log_file_name: String,
    players: Vec<MatchPlayer>,
    runner_config: Arc<GlobalConfig>,
}

pub enum MatchPlayer {
    BotVersion {
        bot: Option<db::bots::Bot>,
        version: db::bots::BotVersion,
    },
    BotSpec {
        spec: Box<dyn BotSpec>,
    },
}

impl RunMatch {
    pub fn from_players(runner_config: Arc<GlobalConfig>, players: Vec<MatchPlayer>) -> Self {
        let log_file_name = format!("{}.log", gen_alphanumeric(16));
        RunMatch {
            runner_config,
            log_file_name,
            players,
        }
    }

    fn into_runner_config(self) -> runner::MatchConfig {
        runner::MatchConfig {
            map_path: PathBuf::from(MAPS_DIR).join("hex.json"),
            map_name: "hex".to_string(),
            log_path: PathBuf::from(MATCHES_DIR).join(&self.log_file_name),
            players: self
                .players
                .into_iter()
                .map(|player| runner::MatchPlayer {
                    bot_spec: match player {
                        MatchPlayer::BotVersion { bot, version } => {
                            bot_version_to_botspec(&self.runner_config, bot.as_ref(), &version)
                        }
                        MatchPlayer::BotSpec { spec } => spec,
                    },
                })
                .collect(),
        }
    }

    pub async fn run(
        self,
        conn_pool: ConnectionPool,
    ) -> QueryResult<(MatchData, JoinHandle<MatchOutcome>)> {
        let match_data = {
            // TODO: it would be nice to get an already-open connection here when possible.
            // Maybe we need an additional abstraction, bundling a connection and connection pool?
            let db_conn = conn_pool.get().await.expect("could not get a connection");
            self.store_in_database(&db_conn)?
        };

        let runner_config = self.into_runner_config();
        let handle = tokio::spawn(run_match_task(conn_pool, runner_config, match_data.base.id));

        Ok((match_data, handle))
    }

    fn store_in_database(&self, db_conn: &PgConnection) -> QueryResult<MatchData> {
        let new_match_data = db::matches::NewMatch {
            state: db::matches::MatchState::Playing,
            log_path: &self.log_file_name,
        };
        let new_match_players = self
            .players
            .iter()
            .map(|p| db::matches::MatchPlayerData {
                code_bundle_id: match p {
                    MatchPlayer::BotVersion { version, .. } => Some(version.id),
                    MatchPlayer::BotSpec { .. } => None,
                },
            })
            .collect::<Vec<_>>();

        db::matches::create_match(&new_match_data, &new_match_players, db_conn)
    }
}

pub fn bot_version_to_botspec(
    runner_config: &Arc<GlobalConfig>,
    bot: Option<&db::bots::Bot>,
    bot_version: &db::bots::BotVersion,
) -> Box<dyn BotSpec> {
    if let Some(code_bundle_path) = &bot_version.code_bundle_path {
        python_docker_bot_spec(runner_config, code_bundle_path)
    } else if let (Some(container_digest), Some(bot)) = (&bot_version.container_digest, bot) {
        Box::new(DockerBotSpec {
            image: format!(
                "{}/{}@{}",
                runner_config.container_registry_url, bot.name, container_digest
            ),
            binds: None,
            argv: None,
            working_dir: None,
        })
    } else {
        // TODO: ideally this would not be possible
        panic!("bad bot version")
    }
}

fn python_docker_bot_spec(
    runner_config: &Arc<GlobalConfig>,
    code_bundle_path: &str,
) -> Box<dyn BotSpec> {
    let code_bundle_rel_path = PathBuf::from(BOTS_DIR).join(code_bundle_path);
    let code_bundle_abs_path = std::fs::canonicalize(&code_bundle_rel_path).unwrap();
    let code_bundle_path_str = code_bundle_abs_path.as_os_str().to_str().unwrap();

    // TODO: it would be good to simplify this configuration
    Box::new(DockerBotSpec {
        image: runner_config.python_runner_image.clone(),
        binds: Some(vec![format!("{}:{}", code_bundle_path_str, "/workdir")]),
        argv: Some(vec!["python".to_string(), "bot.py".to_string()]),
        working_dir: Some("/workdir".to_string()),
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

    let result = MatchResult::Finished {
        winner: outcome.winner.map(|w| (w - 1) as i32), // player numbers in matchrunner start at 1
    };

    db::matches::save_match_result(match_id, result, &conn).expect("could not save match result");

    outcome
}
