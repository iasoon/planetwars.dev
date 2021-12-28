mod commands;
mod match_runner;
mod web;
mod workspace;

pub async fn run() {
    let res = commands::Cli::run().await;
    if let Err(err) = res {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
