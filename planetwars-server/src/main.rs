use std::net::SocketAddr;

extern crate planetwars_server;
extern crate tokio;

#[tokio::main]
async fn main() {
    let app = planetwars_server::app().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
