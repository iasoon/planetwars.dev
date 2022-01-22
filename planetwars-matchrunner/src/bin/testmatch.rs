use std::{env, path::PathBuf};

use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);
    let map_path = args[1].clone();
    _run_match(map_path).await;
}

const IMAGE: &'static str = "python:3.10-slim-buster";

async fn _run_match(map_path: String) {
    let code_dir_path = PathBuf::from("../simplebot");
    let bot_spec = DockerBotSpec {
        image: IMAGE.to_string(),
        code_path: code_dir_path,
        argv: vec!["python".to_string(), "simplebot.py".to_string()],
    };

    run_match(MatchConfig {
        map_path: PathBuf::from(map_path),
        map_name: "hex".to_string(),
        log_path: PathBuf::from("match.log"),
        players: vec![
            MatchPlayer {
                name: "a".to_string(),
                bot_spec: Box::new(bot_spec.clone()),
            },
            MatchPlayer {
                name: "b".to_string(),
                bot_spec: Box::new(bot_spec.clone()),
            },
        ],
    })
    .await;
}
