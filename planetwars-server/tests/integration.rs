use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use planetwars_server::{create_db_pool, create_pw_api, GlobalConfig};
use serde_json::{self, json, Value as JsonValue};
use std::{io, path::Path, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

// Used to serialize tests that access the database.
// TODO: see to what degree we could support transactional testing.
static DB_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

fn create_subdir<P: AsRef<Path>>(base_path: &Path, p: P) -> io::Result<String> {
    let dir_path = base_path.join(p);
    std::fs::create_dir(&dir_path)?;
    let dir_path_string = dir_path.into_os_string().into_string().unwrap();
    Ok(dir_path_string)
}

#[tokio::test]
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
    let app = create_pw_api(config, db_pool);

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/api/bots")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let resp: JsonValue = serde_json::from_slice(&body).unwrap();
    assert_eq!(resp, json!([]));
    Ok(())
}
