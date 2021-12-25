use match_runner::{MatchBot, MatchConfig};
use serde::Deserialize;

mod match_runner;

use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use toml;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "pwcli")]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a match
    RunMatch(RunMatchCommand),
}

#[derive(Parser)]
struct RunMatchCommand {
    /// map name
    map: String,
    /// bot names
    bots: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProjectConfig {
    bots: HashMap<String, BotConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotConfig {
    path: String,
    argv: Vec<String>,
}

pub async fn run() {
    let matches = Cli::parse();
    let res = match matches.command {
        Commands::RunMatch(command) => run_match(command).await,
    };
    if let Err(err) = res {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

async fn run_match(command: RunMatchCommand) -> io::Result<()> {
    let project_dir = env::current_dir().unwrap();

    let config_path = project_dir.join("pw_project.toml");

    let map_path = project_dir.join(format!("maps/{}.json", command.map));

    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
    let log_path = project_dir.join(format!("matches/{}.log", timestamp));

    let config_str = std::fs::read_to_string(config_path).unwrap();
    let project_config: ProjectConfig = toml::from_str(&config_str).unwrap();

    let players = command
        .bots
        .into_iter()
        .map(|bot_name| {
            let bot_config = project_config.bots.get(&bot_name).unwrap().clone();
            let resolved_config = resolve_bot_config(&project_dir, bot_config);
            MatchBot {
                name: bot_name,
                bot_config: resolved_config,
            }
        })
        .collect();

    let match_config = MatchConfig {
        map_path,
        log_path,
        players,
    };

    match_runner::run_match(match_config).await;

    Ok(())
}

fn resolve_bot_config(project_dir: &Path, config: BotConfig) -> BotConfig {
    let mut path = PathBuf::from(project_dir);
    path.push(&config.path);
    BotConfig {
        path: path.to_str().unwrap().to_string(),
        argv: config.argv,
    }
}
