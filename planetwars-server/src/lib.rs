#[macro_use]
extern crate diesel;

pub mod db;
pub mod db_types;
pub mod modules;
pub mod routes;
pub mod schema;
pub mod util;

use std::net::SocketAddr;
use std::ops::Deref;

use bb8::{Pool, PooledConnection};
use bb8_diesel::{self, DieselConnectionManager};
use config::ConfigError;
use diesel::{Connection, PgConnection};
use modules::ranking::run_ranker;
use serde::Deserialize;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::{get, post},
    AddExtensionLayer, Router,
};

// TODO: make these configurable
const BOTS_DIR: &str = "./data/bots";
const MATCHES_DIR: &str = "./data/matches";
const MAPS_DIR: &str = "./data/maps";
const SIMPLEBOT_PATH: &str = "../simplebot/simplebot.py";

type ConnectionPool = bb8::Pool<DieselConnectionManager<PgConnection>>;

pub async fn seed_simplebot(pool: &ConnectionPool) {
    let conn = pool.get().await.expect("could not get database connection");
    // This transaction is expected to fail when simplebot already exists.
    let _res = conn.transaction::<(), diesel::result::Error, _>(|| {
        use db::bots::NewBot;

        let new_bot = NewBot {
            name: "simplebot",
            owner_id: None,
        };

        let simplebot = db::bots::create_bot(&new_bot, &conn)?;

        let simplebot_code =
            std::fs::read_to_string(SIMPLEBOT_PATH).expect("could not read simplebot code");

        modules::bots::save_code_bundle(&simplebot_code, Some(simplebot.id), &conn)?;

        println!("initialized simplebot");

        Ok(())
    });
}

pub type DbPool = Pool<DieselConnectionManager<PgConnection>>;

pub async fn prepare_db(database_url: &str) -> DbPool {
    let manager = DieselConnectionManager::<PgConnection>::new(database_url);
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    seed_simplebot(&pool).await;
    pool
}

pub fn api() -> Router {
    Router::new()
        .route("/register", post(routes::users::register))
        .route("/login", post(routes::users::login))
        .route("/users/me", get(routes::users::current_user))
        .route(
            "/bots",
            get(routes::bots::list_bots).post(routes::bots::create_bot),
        )
        .route("/bots/my_bots", get(routes::bots::get_my_bots))
        .route("/bots/:bot_id", get(routes::bots::get_bot))
        .route(
            "/bots/:bot_id/upload",
            post(routes::bots::upload_code_multipart),
        )
        .route(
            "/matches",
            get(routes::matches::list_matches).post(routes::matches::play_match),
        )
        .route("/matches/:match_id", get(routes::matches::get_match_data))
        .route(
            "/matches/:match_id/log",
            get(routes::matches::get_match_log),
        )
        .route("/leaderboard", get(routes::bots::get_ranking))
        .route("/submit_bot", post(routes::demo::submit_bot))
        .route("/save_bot", post(routes::bots::save_bot))
}

pub fn get_config() -> Result<Configuration, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("configuration.toml"))
        .add_source(config::Environment::with_prefix("PLANETWARS"))
        .build()?
        .try_deserialize()
}

pub async fn run_app() {
    let configuration = get_config().unwrap();
    let db_pool = prepare_db(&configuration.database_url).await;

    tokio::spawn(run_ranker(db_pool.clone()));

    let api_service = Router::new()
        .nest("/api", api())
        .layer(AddExtensionLayer::new(db_pool))
        .into_make_service();

    // TODO: put in config
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    axum::Server::bind(&addr).serve(api_service).await.unwrap();
}

#[derive(Deserialize)]
pub struct Configuration {
    pub database_url: String,
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
