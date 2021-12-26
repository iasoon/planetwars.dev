mod init;
mod run_match;
mod serve;

use clap::{Parser, Subcommand};
use std::io;

#[derive(Parser)]
#[clap(name = "pwcli")]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run() -> io::Result<()> {
        let cli = Self::parse();

        match cli.command {
            Command::Init(command) => command.run().await,
            Command::RunMatch(command) => command.run().await,
            Command::Serve(command) => command.run().await,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new workspace
    Init(init::InitCommand),
    /// Run a match
    RunMatch(run_match::RunMatchCommand),
    /// Host local webserver
    Serve(serve::ServeCommand),
}
