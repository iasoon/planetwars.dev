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
        map_name: "abc".to_string(),
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

#[tokio::test]
async fn player_results() {
    let log_file = tempfile::NamedTempFile::new().unwrap();

    let config = MatchConfig {
        map_name: "abc".to_string(),
        map_path: PathBuf::from("maps/abc.json"),
        log_path: PathBuf::from(log_file.path()),
        players: vec![
            MatchPlayer {
                bot_spec: Box::new(simple_python_docker_bot_spec(
                    "./bots/simplebot",
                    "simplebot.py",
                )),
            },
            MatchPlayer {
                bot_spec: Box::new(simple_python_docker_bot_spec("./bots", "crash_bot.py")),
            },
        ],
    };

    let outcome = run_match(config).await;
    assert_eq!(outcome.player_outcomes.len(), 2);
    assert!(!outcome.player_outcomes[0].crashed);
    assert!(!outcome.player_outcomes[0].had_errors);
    assert!(outcome.player_outcomes[1].crashed);
    assert!(!outcome.player_outcomes[1].had_errors);
}

/// creates a simple match ctx which only holds a single bot
async fn with_bot_match_ctx<B, F>(bot_spec: B, func: F)
where
    F: FnOnce(&mut MatchCtx) -> Pin<Box<dyn '_ + Future<Output = ()>>>,
    B: BotSpec,
{
    let event_bus = Arc::new(Mutex::new(EventBus::new()));
    let (logger, _rx) = mpsc::unbounded_channel();

    let player_handle = bot_spec.run_bot(1, event_bus.clone(), logger.clone()).await;
    let mut players = HashMap::new();
    players.insert(1, player_handle);
    let mut ctx = MatchCtx::new(event_bus, players, logger);

    func(&mut ctx).await;
    ctx.shutdown().await;
}

#[tokio::test]
async fn docker_runner_success() {
    let bot_spec = simple_python_docker_bot_spec("./bots", "echo_bot.py");
    with_bot_match_ctx(bot_spec, |ctx| {
        async move {
            let resp = ctx
                .request(1, b"sup".to_vec(), Duration::from_millis(200))
                .await;

            assert_eq!(resp, Ok(b"sup\n".to_vec()));
        }
        .boxed()
    })
    .await;
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

#[tokio::test]
async fn docker_runner_crash() {
    let bot_spec = simple_python_docker_bot_spec("./bots", "crash_bot.py");
    with_bot_match_ctx(bot_spec, |ctx| {
        async move {
            let resp = ctx
                .request(1, b"sup".to_vec(), Duration::from_millis(200))
                .await;

            assert_eq!(resp, Err(RequestError::BotTerminated));
        }
        .boxed()
    })
    .await;
}

#[tokio::test]
async fn test_long_line() {
    let bot_spec = simple_python_docker_bot_spec("./bots", "echo_bot.py");
    let len = 10 * 2_usize.pow(20); // 10 megabytes - hopefully large enough to cause buffering
    let buf = std::iter::repeat(b'a').take(len).collect::<Vec<u8>>();
    with_bot_match_ctx(bot_spec, |ctx| {
        async move {
            let resp = ctx.request(1, buf, Duration::from_millis(200)).await;

            let resp_bytes = resp.expect("unexpected error");
            assert_eq!(resp_bytes.len(), len + 1);
        }
        .boxed()
    })
    .await;
}
