use shlex;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const BOT_CONFIG_FILENAME: &str = "botconfig.toml";

pub struct WorkspaceBot {
    pub path: PathBuf,
    pub config: BotConfig,
}

impl WorkspaceBot {
    pub fn open(path: &Path) -> io::Result<Self> {
        let config_path = path.join(BOT_CONFIG_FILENAME);
        let config_str = fs::read_to_string(config_path)?;
        let bot_config: BotConfig = toml::from_str(&config_str)?;

        Ok(WorkspaceBot {
            path: path.to_path_buf(),
            config: bot_config,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub run_command: String,
    pub build_command: Option<String>,
}

impl BotConfig {
    // TODO: these commands should not be here
    pub fn get_run_argv(&self) -> Vec<String> {
        // TODO: proper error handling
        shlex::split(&self.run_command)
            .expect("Failed to parse bot run command. Check for unterminated quotes.")
    }

    pub fn get_build_argv(&self) -> Option<Vec<String>> {
        // TODO: proper error handling
        self.build_command.as_ref().map(|cmd| {
            shlex::split(cmd)
                .expect("Failed to parse bot build command. Check for unterminated quotes.")
        })
    }
}
