use std::net::SocketAddr;

extern crate mozaic4_backend;
extern crate tokio;

#[tokio::main]
async fn main() {
    let app = mozaic4_backend::app().await;

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
