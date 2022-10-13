use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    Router,
};
use diesel::{PgConnection, RunQueryDsl};
use planetwars_server::{create_db_pool, create_pw_api, db, modules, DbPool, GlobalConfig};
use serde_json::{self, json, Value as JsonValue};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
    time::Duration,
};
use tempfile::TempDir;
use tower::Service;

// Used to serialize tests that access the database.
// TODO: see to what degree we could support transactional testing.
static DB_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

fn create_subdir<P: AsRef<Path>>(base_path: &Path, p: P) -> io::Result<String> {
    let dir_path = base_path.join(p);
    std::fs::create_dir(&dir_path)?;
    let dir_path_string = dir_path.into_os_string().into_string().unwrap();
    Ok(dir_path_string)
}

fn clear_database(conn: &mut PgConnection) {
    diesel::sql_query(
        "TRUNCATE TABLE
        bots,
        bot_versions,
        maps,
        matches,
        match_players,
        ratings,
        sessions,
        users",
    )
    .execute(conn)
    .expect("failed to clear database");
}

/// Setup a simple text fixture, having simplebot and the hex map.
/// This is enough to run a simple match.
fn setup_simple_fixture(db_conn: &mut PgConnection, config: &GlobalConfig) {
    let bot = db::bots::create_bot(
        &db::bots::NewBot {
            owner_id: None,
            name: "simplebot",
        },
        db_conn,
    )
    .expect("could not create simplebot");

    let simplebot_code = std::fs::read_to_string("../simplebot/simplebot.py")
        .expect("could not read simplebot code");
    let _bot_version =
        modules::bots::save_code_string(&simplebot_code, Some(bot.id), db_conn, &config)
            .expect("could not save bot version");

    std::fs::copy(
        "../maps/hex.json",
        PathBuf::from(&config.maps_directory).join("hex.json"),
    )
    .expect("could not copy map");
    db::maps::create_map(
        db::maps::NewMap {
            name: "hex",
            file_path: "hex.json",
        },
        db_conn,
    )
    .expect("could not save map");
}

struct TestApp<'a> {
    // exclusive connection to the test database
    #[allow(dead_code)]
    db_guard: parking_lot::MutexGuard<'a, ()>,
    db_pool: DbPool,

    // temporary data directory
    #[allow(dead_code)]
    data_dir: TempDir,

    config: Arc<GlobalConfig>,
}

impl<'a> TestApp<'a> {
    async fn create() -> io::Result<TestApp<'a>> {
        let data_dir = TempDir::new().expect("failed to create temp dir");

        let config = Arc::new(GlobalConfig {
            database_url: "postgresql://planetwars:planetwars@localhost/planetwars-test"
                .to_string(),
            python_runner_image: "python:3.10-slim-buster".to_string(),
            container_registry_url: "localhost:9001".to_string(),
            root_url: "localhost:3000".to_string(),
            bots_directory: create_subdir(data_dir.path(), "bots")?,
            match_logs_directory: create_subdir(data_dir.path(), "matches")?,
            maps_directory: create_subdir(data_dir.path(), "maps")?,
            registry_directory: create_subdir(data_dir.path(), "registry")?,
            registry_admin_password: "secret_admin_password".to_string(),
            ranker_enabled: false,
        });
        let db_guard = DB_LOCK.lock();
        let db_pool = create_db_pool(&config).await;

        Ok(TestApp {
            db_guard,
            config,
            data_dir,
            db_pool,
        })
    }

    async fn with_db_conn<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut PgConnection) -> R,
    {
        let mut db_conn = self
            .db_pool
            .get()
            .await
            .expect("could not get db connection");
        function(&mut db_conn)
    }

    async fn play_public_match(&self, bot_names: &[&str], map_name: &str) {
        let mut conn = self.db_pool.get().await.unwrap();
        let map = db::maps::find_map_by_name(map_name, &mut conn).unwrap();

        let mut bots = Vec::new();
        for bot_name in bot_names.iter() {
            let (bot, bot_version) =
                db::bots::find_bot_with_version_by_name(bot_name, &mut conn).unwrap();
            bots.push((bot, bot_version));
        }

        modules::ranking::play_ranked_match(self.config.clone(), map, bots, self.db_pool.clone())
            .await;
    }
}

async fn poll_match(app: &mut Router, match_id: &str) -> io::Result<Poll<JsonValue>> {
    let response = app
        .call(
            Request::builder()
                .method(http::Method::GET)
                .header("Content-Type", "application/json")
                .uri(format!("/api/matches/{}", match_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resp: JsonValue = serde_json::from_slice(&body).unwrap();

    match resp["state"].as_str() {
        Some("Playing") => Ok(Poll::Pending),
        Some("Finished") => Ok(Poll::Ready(resp)),
        // TODO: replace with err
        value => panic!("got unexpected match state {:?}", value),
    }
}

async fn poll_match_until_complete(app: &mut Router, match_id: &str) -> io::Result<JsonValue> {
    let poll_interval = Duration::from_millis(100);
    let mut interval = tokio::time::interval(poll_interval);
    loop {
        interval.tick().await;
        match poll_match(app, match_id).await {
            Ok(Poll::Ready(result)) => return Ok(result),
            Ok(Poll::Pending) => (),
            Err(err) => return Err(err),
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_submit_bot() -> io::Result<()> {
    let test_app = TestApp::create().await.unwrap();
    test_app
        .with_db_conn(|db_conn| {
            clear_database(db_conn);
            setup_simple_fixture(db_conn, &test_app.config);
        })
        .await;

    let mut app = create_pw_api(test_app.config, test_app.db_pool);

    let simplebot_code = std::fs::read_to_string("../simplebot/simplebot.py")
        .expect("could not read simplebot code");

    let payload = json!({
        "code": simplebot_code,
    });
    let response = app
        .call(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .uri("/api/submit_bot")
                .body(serde_json::to_vec(&payload).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resp: JsonValue = serde_json::from_slice(&body).unwrap();

    let match_id = &resp["match"]["id"].as_i64().unwrap();
    let _match_result = tokio::time::timeout(
        Duration::from_secs(10),
        poll_match_until_complete(&mut app, &match_id.to_string()),
    )
    .await
    .expect("fetching match result timed out")
    .expect("failed to get match result");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_sign_up_and_create_bot() -> io::Result<()> {
    let test_app = TestApp::create().await.unwrap();
    test_app
        .with_db_conn(|db_conn| {
            clear_database(db_conn);
            setup_simple_fixture(db_conn, &test_app.config);
        })
        .await;

    let mut app = create_pw_api(test_app.config, test_app.db_pool);

    // Registration
    let credentials = json!({
        "username": "piepkonijn",
        "password": "123geheim",
    });
    let response = app
        .call(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .uri("/api/register")
                .body(serde_json::to_vec(&credentials).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Login
    let response = app
        .call(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .uri("/api/login")
                .body(serde_json::to_vec(&credentials).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let session_token = response.headers()["Token"].to_str().unwrap().clone();

    // save bot
    let simplebot_code = std::fs::read_to_string("../simplebot/simplebot.py")
        .expect("could not read simplebot code");

    let payload = json!({
        "bot_name": "testbot",
        "code": simplebot_code,
    });

    let response = app
        .call(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", session_token))
                .uri("/api/save_bot")
                .body(serde_json::to_vec(&payload).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // launch a match against the new bot
    let payload = json!({
        "code": simplebot_code,
        // TODO: how can we test that this bot is acutally being selected?
        "opponent_name": "testbot",
    });
    let response = app
        .call(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .uri("/api/submit_bot")
                .body(serde_json::to_vec(&payload).unwrap().into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resp: JsonValue = serde_json::from_slice(&body).unwrap();

    let match_id = &resp["match"]["id"].as_i64().unwrap();
    let _match_result = tokio::time::timeout(
        Duration::from_secs(10),
        poll_match_until_complete(&mut app, &match_id.to_string()),
    )
    .await
    .expect("fetching match result timed out")
    .expect("failed to get match result");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_list_matches_with_errors() -> io::Result<()> {
    let test_app = TestApp::create().await.unwrap();
    test_app
        .with_db_conn(|db_conn| {
            clear_database(db_conn);
            setup_simple_fixture(db_conn, &test_app.config);

            let bot = db::bots::create_bot(
                &db::bots::NewBot {
                    owner_id: None,
                    name: "testbot",
                },
                db_conn,
            )
            .expect("could not create bot");

            let failing_code = "import sys; sys.exit(1)";

            let _bot_version = modules::bots::save_code_string(
                failing_code,
                Some(bot.id),
                db_conn,
                &test_app.config,
            )
            .expect("could not save bot version");
        })
        .await;

    test_app
        .play_public_match(&["simplebot", "testbot"], "hex")
        .await;

    let mut app = create_pw_api(test_app.config, test_app.db_pool);

    let response = app
        .call(
            Request::builder()
                .method(http::Method::GET)
                .header("Content-Type", "application/json")
                .uri(format!("/api/matches?bot=testbot"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resp: JsonValue = serde_json::from_slice(&body).unwrap();

    let matches = resp["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0]["players"][0]["had_errors"].as_bool(),
        Some(false)
    );
    assert_eq!(matches[0]["players"][1]["had_errors"].as_bool(), Some(true));

    Ok(())
}
