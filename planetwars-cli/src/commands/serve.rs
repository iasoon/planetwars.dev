use std::env;
use std::io;

use clap::Parser;

use crate::web;

#[derive(Parser)]
pub struct ServeCommand;

impl ServeCommand {
    pub async fn run(self) -> io::Result<()> {
        let workspace_root = env::current_dir().unwrap();

        web::run(workspace_root).await;
        Ok(())
    }
}
