use std::io;

use clap::Parser;
use planetwars_matchrunner::{run_match, MatchConfig, MatchPlayer};

use crate::workspace::Workspace;
#[derive(Parser)]
pub struct RunMatchCommand {
    /// map name
    map: String,
    /// bot names
    bots: Vec<String>,
}

impl RunMatchCommand {
    pub async fn run(self) -> io::Result<()> {
        let workspace = Workspace::open_current_dir()?;

        let map_path = workspace.map_path(&self.map);
        let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
        let log_path = workspace.match_path(&format!("{}-{}", &self.map, &timestamp));

        let mut players = Vec::new();
        for bot_name in &self.bots {
            let bot = workspace.get_bot(&bot_name)?;
            players.push(MatchPlayer {
                name: bot_name.clone(),
                path: bot.path.clone(),
                argv: bot.config.get_run_argv(),
            });
        }

        let match_config = MatchConfig {
            map_name: self.map,
            map_path,
            log_path: log_path.clone(),
            players,
        };

        run_match(match_config).await;
        println!("match completed successfully");
        // TODO: maybe print the match result as well?

        let relative_path = match log_path.strip_prefix(&workspace.root_path) {
            Ok(path) => path.to_str().unwrap(),
            Err(_) => log_path.to_str().unwrap(),
        };
        println!("wrote match log to {}", relative_path);
        Ok(())
    }
}
