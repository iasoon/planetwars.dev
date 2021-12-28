use std::io;

use clap::Parser;

use crate::web;
use crate::workspace::Workspace;

#[derive(Parser)]
pub struct ServeCommand;

impl ServeCommand {
    pub async fn run(self) -> io::Result<()> {
        let workspace = Workspace::open_current_dir()?;
        web::run(workspace).await;
        Ok(())
    }
}
