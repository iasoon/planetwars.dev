use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use self::bot::WorkspaceBot;

const WORKSPACE_CONFIG_FILENAME: &str = "pw_workspace.toml";

pub mod bot;

pub struct Workspace {
    root_path: PathBuf,
    config: WorkspaceConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceConfig {
    paths: WorkspacePaths,
    bots: HashMap<String, BotEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspacePaths {
    maps_dir: PathBuf,
    matches_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotEntry {
    path: PathBuf,
}

impl Workspace {
    pub fn open(root_path: &Path) -> io::Result<Workspace> {
        let config_path = root_path.join(WORKSPACE_CONFIG_FILENAME);
        let config_str = fs::read_to_string(config_path)?;
        let workspace_config: WorkspaceConfig = toml::from_str(&config_str)?;

        Ok(Workspace {
            root_path: root_path.to_path_buf(),
            config: workspace_config,
        })
    }

    pub fn open_current_dir() -> io::Result<Workspace> {
        Workspace::open(&env::current_dir()?)
    }

    pub fn maps_dir(&self) -> PathBuf {
        self.root_path.join(&self.config.paths.maps_dir)
    }

    pub fn map_path(&self, map_name: &str) -> PathBuf {
        self.maps_dir().join(format!("{}.json", map_name))
    }

    pub fn matches_dir(&self) -> PathBuf {
        self.root_path.join(&self.config.paths.matches_dir)
    }

    pub fn match_path(&self, match_name: &str) -> PathBuf {
        self.matches_dir().join(format!("{}.log", match_name))
    }

    pub fn get_bot(&self, bot_name: &str) -> io::Result<WorkspaceBot> {
        let bot_entry = self.config.bots.get(bot_name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no such bot: {}", bot_name),
            )
        })?;
        WorkspaceBot::open(&self.root_path.join(&bot_entry.path))
    }
}
