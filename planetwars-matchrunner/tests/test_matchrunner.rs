use futures::{Future, FutureExt};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

use planetwars_matchrunner::docker_runner::DockerBotSpec;
use planetwars_matchrunner::match_context::{EventBus, MatchCtx, RequestError};
use planetwars_matchrunner::BotSpec;
use planetwars_matchrunner::{run_match, MatchConfig, MatchPlayer};

const PYTHON_IMAGE: &str = "python:3.10-slim-buster";

fn simple_python_docker_bot_spec(source_dir: &str, file_name: &str) -> DockerBotSpec {
    let source_dir_path = std::fs::canonicalize(source_dir).unwrap();
    let source_dir_path_str = source_dir_path.as_os_str().to_str().unwrap();

    DockerBotSpec {
        image: PYTHON_IMAGE.to_string(),
        binds: Some(vec![format!("{}:{}", source_dir_path_str, "/workdir")]),
        argv: Some(vec!["python".to_string(), file_name.to_string()]),
        working_dir: Some("/workdir".to_string()),
        pull: false,
        credentials: None,
    }
}

#[tokio::test]
async fn match_does_run() {
    let bot = simple_python_docker_bot_spec("./bots/simplebot", "simplebot.py");

    let log_file = tempfile::NamedTempFile::new().unwrap();

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

    tokio::time::sleep(Duration::from_secs(1)).await
}

async fn match_ctx_from_bot_spec<B: BotSpec>(bot_spec: B) -> MatchCtx {
    let event_bus = Arc::new(Mutex::new(EventBus::new()));
    let (logger, _rx) = mpsc::unbounded_channel();

    let player_handle = bot_spec.run_bot(1, event_bus.clone(), logger.clone()).await;
    let mut players = HashMap::new();
    players.insert(1, player_handle);
    MatchCtx::new(event_bus, players, logger)
}

/// creates a simple match ctx which only holds a single bot
async fn with_bot_match_ctx<B, F>(bot_spec: B, func: F)
where
    F: FnOnce(&mut MatchCtx) -> Pin<Box<dyn '_ + Future<Output = ()>>>,
    B: BotSpec,
{
    let mut ctx = match_ctx_from_bot_spec(bot_spec).await;
    func(&mut ctx).await;
    ctx.shutdown().await;
}

#[tokio::test]
async fn docker_runner_timeout() {
    let bot_spec = simple_python_docker_bot_spec("./bots", "timeout_bot.py");
    with_bot_match_ctx(bot_spec, |ctx| {
        async move {
            let resp = ctx
                .request(1, b"sup".to_vec(), Duration::from_millis(200))
                .await;

            assert_eq!(resp, Err(RequestError::Timeout));
        }
        .boxed()
    })
    .await;
}
