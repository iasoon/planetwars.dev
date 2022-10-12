extern crate planetwars_server;
extern crate tokio;

use clap::Parser;
use planetwars_server::db;
use planetwars_server::{create_db_pool, get_config};

#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    SetPassword(SetPassword),
}

impl Action {
    async fn run(self) {
        match self {
            Action::SetPassword(set_password) => set_password.run().await,
        }
    }
}

#[derive(clap::Parser)]
struct SetPassword {
    #[clap(value_parser)]
    username: String,

    #[clap(value_parser)]
    new_password: String,
}

impl SetPassword {
    async fn run(self) {
        let global_config = get_config().unwrap();
        let pool = create_db_pool(&global_config).await;

        let mut conn = pool.get().await.expect("could not get database connection");
        let credentials = db::users::Credentials {
            username: &self.username,
            password: &self.new_password,
        };
        db::users::set_user_password(credentials, &mut conn).expect("could not set password");
    }
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    args.action.run().await;
}
