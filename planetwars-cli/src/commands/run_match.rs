use std::env;
use std::io;

use clap::Parser;

use crate::match_runner;
use crate::match_runner::MatchBot;
use crate::match_runner::MatchConfig;
use crate::resolve_bot_config;
use crate::WorkspaceConfig;

#[derive(Parser)]
pub struct RunMatchCommand {
    /// map name
    map: String,
    /// bot names
    bots: Vec<String>,
}

impl RunMatchCommand {
    pub async fn run(self) -> io::Result<()> {
        let workspace_root = env::current_dir().unwrap();

        let config_path = workspace_root.join("pw_workspace.toml");

        let map_path = workspace_root.join(format!("maps/{}.json", self.map));

        let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
        let log_path = workspace_root.join(format!("matches/{}.log", timestamp));

        let config_str = std::fs::read_to_string(config_path).unwrap();
        let workspace_config: WorkspaceConfig = toml::from_str(&config_str).unwrap();

        let players = self
            .bots
            .into_iter()
            .map(|bot_name| {
                let bot_config = workspace_config.bots.get(&bot_name).unwrap().clone();
                let resolved_config = resolve_bot_config(&workspace_root, bot_config);
                MatchBot {
                    name: bot_name,
                    bot_config: resolved_config,
                }
            })
            .collect();

        let match_config = MatchConfig {
            map_name: self.map,
            map_path,
            log_path,
            players,
        };

        match_runner::run_match(match_config).await;
        println!("match completed successfully");
        // TODO: don't hardcode match path.
        // maybe print the match result as well?
        println!("wrote match log to matches/{}.log", timestamp);
        Ok(())
    }
}
