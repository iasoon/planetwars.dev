use clap::Parser;
use std::io;
use tokio::process;

use crate::workspace::Workspace;

#[derive(Parser)]
pub struct BuildCommand {
    /// Name of the bot to build
    bot: String,
}

impl BuildCommand {
    pub async fn run(self) -> io::Result<()> {
        let workspace = Workspace::open_current_dir()?;
        let bot = workspace.get_bot(&self.bot)?;
        if let Some(argv) = bot.config.get_build_argv() {
            process::Command::new(&argv[0])
                .args(&argv[1..])
                .current_dir(&bot.path)
                .spawn()?
                .wait()
                .await?;
        }
        Ok(())
    }
}
