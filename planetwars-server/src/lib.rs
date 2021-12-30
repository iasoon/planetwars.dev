#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;

pub mod db;
pub mod routes;
pub mod schema;

use std::ops::Deref;

use axum;
use bb8::PooledConnection;
use bb8_diesel::{self, DieselConnectionManager};
use diesel::PgConnection;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::{get, post},
    AddExtensionLayer, Router,
};

async fn index_handler() -> &'static str {
    "Hello, world!"
}

type ConnectionPool = bb8::Pool<DieselConnectionManager<PgConnection>>;

pub async fn app() -> Router {
    let database_url = "postgresql://planetwars:planetwars@localhost/planetwars";
    let manager = DieselConnectionManager::<PgConnection>::new(database_url);
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/users/register", post(routes::users::register))
        .route("/users/login", post(routes::users::login))
        .route("/users/me", get(routes::users::current_user))
        .route("/bots", post(routes::bots::create_bot))
        .route("/bots/:bot_id", get(routes::bots::get_bot))
        .route("/bots/:bot_id/upload", post(routes::bots::upload_bot_code))
        .layer(AddExtensionLayer::new(pool));
    app
}

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
pub struct DatabaseConnection(PooledConnection<'static, DieselConnectionManager<PgConnection>>);

impl Deref for DatabaseConnection {
    type Target = PooledConnection<'static, DieselConnectionManager<PgConnection>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<ConnectionPool>::from_request(req)
            .await
            .map_err(internal_error)?;

        let conn = pool.get_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
