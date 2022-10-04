use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use diesel::{PgConnection, RunQueryDsl};
use planetwars_server::{create_db_pool, create_pw_api, db, modules, GlobalConfig};
use serde_json::{self, json, Value as JsonValue};
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
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

fn clear_database(conn: &PgConnection) {
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

#[tokio::test(flavor = "multi_thread")]
async fn test_application() -> io::Result<()> {
    let _db_guard = DB_LOCK.lock();
    let data_dir = TempDir::new().expect("failed to create temp dir");
    let config = Arc::new(GlobalConfig {
        database_url: "postgresql://planetwars:planetwars@localhost/planetwars-test".to_string(),
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
    let db_pool = create_db_pool(&config).await;
    {
        let db_conn = db_pool.get().await.expect("failed to get db connection");
        clear_database(&db_conn);

        let bot = db::bots::create_bot(
            &db::bots::NewBot {
                owner_id: None,
                name: "simplebot",
            },
            &db_conn,
        )
        .expect("could not create simplebot");

        let simplebot_code = std::fs::read_to_string("../simplebot/simplebot.py")
            .expect("could not read simplebot code");
        let _bot_version =
            modules::bots::save_code_string(&simplebot_code, Some(bot.id), &db_conn, &config)
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
            &db_conn,
        )
        .expect("could not save map");
    }
    let mut app = create_pw_api(config, db_pool);

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

    let match_id = &resp["match"]["id"];
    let mut num_tries = 0;
    loop {
        num_tries += 1;
        assert!(num_tries <= 100, "time limit exceeded");
        tokio::time::sleep(Duration::from_millis(100)).await;

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
            Some("Playing") => (),             // continue,
            Some("Finished") => return Ok(()), // success
            value => panic!("got unexpected match state {:?}", value),
        }
    }
}
