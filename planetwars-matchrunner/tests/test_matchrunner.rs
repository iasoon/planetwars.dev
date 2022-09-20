use std::io::BufRead;
use std::path::PathBuf;

use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};

const PYTHON_IMAGE: &str = "python:3.10-slim-buster";

#[tokio::test]
async fn match_does_run() {
    let simplebot_path = std::fs::canonicalize("bots/simplebot").unwrap();
    let simplebot_path_str = simplebot_path.as_os_str().to_str().unwrap();

    let log_file = tempfile::NamedTempFile::new().unwrap();

    let bot = DockerBotSpec {
        image: PYTHON_IMAGE.to_string(),
        binds: Some(vec![format!("{}:{}", simplebot_path_str, "/workdir")]),
        argv: Some(vec!["python".to_string(), "simplebot.py".to_string()]),
        working_dir: Some("/workdir".to_string()),
        pull: false,
        credentials: None,
    };
    let config = MatchConfig {
        map_name: "hex".to_string(),
        map_path: PathBuf::from("maps/abc.json"),
        log_path: PathBuf::from(log_file.path()),
        players: vec![
            MatchPlayer {
                bot_spec: Box::new(bot.clone()),
            },
            MatchPlayer {
                bot_spec: Box::new(bot.clone()),
            },
        ],
    };

    run_match(config).await;

    let line_count = std::io::BufReader::new(log_file.as_file()).lines().count();
    assert!(line_count > 0);
}
