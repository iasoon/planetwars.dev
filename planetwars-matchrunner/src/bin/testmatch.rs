use std::{env, path::PathBuf};

use planetwars_matchrunner::{docker_runner::DockerBotSpec, run_match, MatchConfig, MatchPlayer};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);
    let map_path = args[1].clone();
    _run_match(map_path).await;
}

const IMAGE: &str = "python:3.10-slim-buster";

async fn _run_match(map_path: String) {
    run_match(MatchConfig {
        map_path: PathBuf::from(map_path),
        map_name: "hex".to_string(),
        log_path: PathBuf::from("match.log"),
        players: vec![
            MatchPlayer {
                name: "a".to_string(),
                bot_spec: Box::new(DockerBotSpec {
                    image: IMAGE.to_string(),
                    // code_path: PathBuf::from("../simplebot"),
                    code_path: PathBuf::from("./bots/simplebot"),
                    argv: vec!["python".to_string(), "simplebot.py".to_string()],
                }),
            },
            MatchPlayer {
                name: "b".to_string(),
                bot_spec: Box::new(DockerBotSpec {
                    image: IMAGE.to_string(),
                    code_path: PathBuf::from("./bots/broken_bot"),
                    argv: vec!["python".to_string(), "bot.py".to_string()],
                }),
            },
        ],
    })
    .await;

    // TODO: use a joinhandle to wait for the logger to finish
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
