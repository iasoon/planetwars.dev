extern crate planetwars_server;
extern crate tokio;

#[tokio::main]
async fn main() {
    planetwars_server::run_app().await;
}
