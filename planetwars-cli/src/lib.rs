use serde::Deserialize;

mod commands;
mod match_runner;
mod web;

use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct WorkspaceConfig {
    bots: HashMap<String, BotConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    path: String,
    argv: Vec<String>,
}

pub async fn run() {
    let res = commands::Cli::run().await;
    if let Err(err) = res {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn resolve_bot_config(workspace_dir: &Path, config: BotConfig) -> BotConfig {
    let mut path = PathBuf::from(workspace_dir);
    path.push(&config.path);
    BotConfig {
        path: path.to_str().unwrap().to_string(),
        argv: config.argv,
    }
}
