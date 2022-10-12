#[macro_use]
extern crate diesel;

pub mod db;
pub mod db_types;
pub mod modules;
pub mod routes;
pub mod schema;
pub mod util;

use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, net::SocketAddr};

use bb8::{Pool, PooledConnection};
use bb8_diesel::{self, DieselConnectionManager};
use config::ConfigError;
use diesel::{Connection, PgConnection};
use modules::client_api::run_client_api;
use modules::ranking::run_ranker;
use modules::registry::registry_service;
use serde::{Deserialize, Serialize};

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use tower_http::compression::CompressionLayer;

type ConnectionPool = bb8::Pool<DieselConnectionManager<PgConnection>>;

// this should probably be modularized a bit as the config grows
#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    /// url for the postgres database
    pub database_url: String,

    /// which image to use for running python bots
    pub python_runner_image: String,

    /// url for the internal container registry
    /// this will be used when running bots
    pub container_registry_url: String,

    /// webserver root url, used to construct links
    pub root_url: String,

    /// directory where bot code will be stored
    pub bots_directory: String,
    /// directory where match logs will be stored
    pub match_logs_directory: String,
    /// directory where map files will be stored
    pub maps_directory: String,

    /// base directory for registry data
    pub registry_directory: String,
    /// secret admin password for internal docker login
    /// used to pull bots when running matches
    pub registry_admin_password: String,

    /// Whether to run the ranker
    pub ranker_enabled: bool,
}

// TODO: do we still need this? Is there a better way?
const SIMPLEBOT_PATH: &str = "../simplebot/simplebot.py";

pub async fn seed_simplebot(config: &GlobalConfig, pool: &ConnectionPool) {
    let mut conn = pool.get().await.expect("could not get database connection");
    // This transaction is expected to fail when simplebot already exists.
    let _res = conn.transaction::<(), diesel::result::Error, _>(|conn| {
        use db::bots::NewBot;

        let new_bot = NewBot {
            name: "simplebot",
            owner_id: None,
        };

        let simplebot = db::bots::create_bot(&new_bot, conn)?;

        let simplebot_code =
            std::fs::read_to_string(SIMPLEBOT_PATH).expect("could not read simplebot code");

        modules::bots::save_code_string(&simplebot_code, Some(simplebot.id), conn, config)?;

        println!("initialized simplebot");

        Ok(())
    });
}

pub type DbPool = Pool<DieselConnectionManager<PgConnection>>;

pub async fn create_db_pool(config: &GlobalConfig) -> DbPool {
    let manager = DieselConnectionManager::<PgConnection>::new(&config.database_url);
    bb8::Pool::builder().build(manager).await.unwrap()
}

// create all directories required for further operation
fn init_directories(config: &GlobalConfig) -> std::io::Result<()> {
    fs::create_dir_all(&config.bots_directory)?;
    fs::create_dir_all(&config.maps_directory)?;
    fs::create_dir_all(&config.match_logs_directory)?;

    let registry_path = PathBuf::from(&config.registry_directory);
    fs::create_dir_all(registry_path.join("sha256"))?;
    fs::create_dir_all(registry_path.join("manifests"))?;
    fs::create_dir_all(registry_path.join("uploads"))?;
    Ok(())
}

// just the routes, without connecting state
fn api() -> Router {
    Router::new()
        .route("/register", post(routes::users::register))
        .route("/login", post(routes::users::login))
        .route("/users/me", get(routes::users::current_user))
        .route("/users/:user/bots", get(routes::bots::get_user_bots))
        .route(
            "/bots",
            get(routes::bots::list_bots).post(routes::bots::create_bot),
        )
        .route("/bots/:bot_name", get(routes::bots::get_bot))
        .route("/bots/:bot_name/stats", get(routes::bots::get_bot_stats))
        .route(
            "/bots/:bot_name/upload",
            post(routes::bots::upload_code_multipart),
        )
        .route("/code/:version_id", get(routes::bots::get_code))
        .route("/matches", get(routes::matches::list_recent_matches))
        .route("/matches/:match_id", get(routes::matches::get_match_data))
        .route(
            "/matches/:match_id/log",
            get(routes::matches::get_match_log),
        )
        .route("/maps", get(routes::maps::list_maps))
        .route("/leaderboard", get(routes::bots::get_ranking))
        .route("/submit_bot", post(routes::demo::submit_bot))
        .route("/save_bot", post(routes::bots::save_bot))
}

// full service
pub fn create_pw_api(global_config: Arc<GlobalConfig>, db_pool: DbPool) -> Router {
    Router::new()
        .nest("/api", api())
        .layer(Extension(db_pool))
        .layer(Extension(global_config))
        .layer(CompressionLayer::new())
}

pub fn get_config() -> Result<GlobalConfig, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("configuration.toml"))
        .add_source(config::Environment::with_prefix("PLANETWARS"))
        .build()?
        .try_deserialize()
}

async fn run_registry(config: Arc<GlobalConfig>, db_pool: DbPool) {
    // TODO: put in config
    let addr = SocketAddr::from(([127, 0, 0, 1], 9001));

    axum::Server::bind(&addr)
        .serve(
            registry_service()
                .layer(Extension(db_pool))
                .layer(Extension(config))
                .into_make_service(),
        )
        .await
        .unwrap();
}

pub async fn run_app() {
    let global_config = Arc::new(get_config().unwrap());
    let db_pool = create_db_pool(&global_config).await;
    seed_simplebot(&global_config, &db_pool).await;
    init_directories(&global_config).unwrap();

    if global_config.ranker_enabled {
        tokio::spawn(run_ranker(global_config.clone(), db_pool.clone()));
    }
    tokio::spawn(run_registry(global_config.clone(), db_pool.clone()));
    tokio::spawn(run_client_api(global_config.clone(), db_pool.clone()));

    // TODO: put in config
    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));

    let pw_api_service = create_pw_api(global_config, db_pool).into_make_service();
    axum::Server::bind(&addr)
        .serve(pw_api_service)
        .await
        .unwrap();
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

impl DerefMut for DatabaseConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
