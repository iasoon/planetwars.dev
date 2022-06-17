use axum::body::Body;
use axum::extract::{BodyStream, Path, Query};
use axum::handler::Handler;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, head, post, put};
use axum::Router;
use hyper::StatusCode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use crate::util::gen_alphanumeric;

const REGISTRY_PATH: &'static str = "./data/registry";
pub fn registry_service() -> Router {
    Router::new()
        .nest("/v2", registry_api_v2())
        .fallback(fallback.into_service())
}

fn registry_api_v2() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/:name/blobs/:digest", head(blob_check).get(blob_check))
        .route("/:name/blobs/uploads/", post(blob_upload))
        .route(
            "/:name/blobs/uploads/:uuid",
            put(put_handler).patch(handle_upload),
        )
        .route("/:name/manifests/:reference", put(put_manifest))
}

async fn fallback(request: axum::http::Request<Body>) -> impl IntoResponse {
    // for debugging
    println!("no route for {} {}", request.method(), request.uri());
    StatusCode::NOT_FOUND
}

// root should return 200 OK to confirm api compliance
async fn root_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Docker-Distribution-API-Version", "registry/2.0")
        .body(Body::empty())
        .unwrap()
}

#[derive(Serialize)]
pub struct RegistryErrors {
    errors: Vec<RegistryError>,
}

#[derive(Serialize)]
pub struct RegistryError {
    code: String,
    message: String,
    detail: serde_json::Value,
}

async fn blob_check(
    Path((_repository_name, raw_digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let digest = raw_digest.strip_prefix("sha256:").unwrap();
    let blob_path = PathBuf::from(REGISTRY_PATH).join("sha256").join(&digest);
    if blob_path.exists() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn blob_upload(Path(repository_name): Path<String>) -> impl IntoResponse {
    // let value = json!({
    //     "errors": [
    //         {
    //             "code": "UNSUPPORTED",
    //             "message": "not implemented yet lol",
    //         }
    //     ]
    // });

    let uuid = gen_alphanumeric(16);
    tokio::fs::File::create(PathBuf::from(REGISTRY_PATH).join("uploads").join(&uuid))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(
            "Location",
            format!("/v2/{}/blobs/uploads/{}", repository_name, uuid),
        )
        .header("Docker-Upload-UUID", uuid)
        .header("Range", "bytes=0-0")
        .body(Body::empty())
        .unwrap()
}

use futures::StreamExt;

async fn handle_upload(
    Path((repository_name, uuid)): Path<(String, String)>,
    mut stream: BodyStream,
) -> impl IntoResponse {
    // let content_length = headers.get("Content-Length").unwrap();
    // let content_range = headers.get("Content-Range").unwrap();
    // let content_type = headers.get("Content-Type").unwrap();
    // assert!(content_type == "application/octet-stream");
    let mut len = 0;
    let upload_path = PathBuf::from(REGISTRY_PATH).join("uploads").join(&uuid);
    let mut file = tokio::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .append(true)
        .create(false)
        .open(upload_path)
        .await
        .unwrap();
    while let Some(Ok(chunk)) = stream.next().await {
        let n_bytes = file.write(&chunk).await.unwrap();
        len += n_bytes;
    }

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(
            "Location",
            format!("/v2/{}/blobs/uploads/{}", repository_name, uuid),
        )
        .header("Docker-Upload-UUID", uuid)
        .header("Range", format!("0-{}", len))
        .body(Body::empty())
        .unwrap()
}

use serde::Deserialize;
#[derive(Deserialize)]
struct UploadParams {
    digest: String,
}

async fn put_handler(
    Path((repository_name, uuid)): Path<(String, String)>,
    Query(params): Query<UploadParams>,
    mut stream: BodyStream,
) -> impl IntoResponse {
    let mut _len = 0;
    let upload_path = PathBuf::from(REGISTRY_PATH).join("uploads").join(&uuid);
    let mut file = tokio::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .append(true)
        .create(false)
        .open(&upload_path)
        .await
        .unwrap();

    while let Some(Ok(chunk)) = stream.next().await {
        let n_bytes = file.write(&chunk).await.unwrap();
        _len += n_bytes;
    }
    let digest = params.digest.strip_prefix("sha256:").unwrap();
    // TODO: check the digest
    let target_path = PathBuf::from(REGISTRY_PATH).join("sha256").join(&digest);
    tokio::fs::rename(&upload_path, &target_path).await.unwrap();

    Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/blobs/{}", repository_name, digest),
        )
        .header("Docker-Upload-UUID", uuid)
        // .header("Range", format!("0-{}", len))
        .header("Docker-Content-Digest", digest)
        .body(Body::empty())
        .unwrap()
}

async fn put_manifest(
    Path((repository_name, reference)): Path<(String, String)>,
    mut stream: BodyStream,
) -> impl IntoResponse {
    let repository_dir = PathBuf::from(REGISTRY_PATH)
        .join("manifests")
        .join(&repository_name);

    tokio::fs::create_dir_all(&repository_dir).await.unwrap();

    let mut hasher = Sha256::new();
    {
        let manifest_path = repository_dir.join(&reference).with_extension("json");
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&manifest_path)
            .await
            .unwrap();
        while let Some(Ok(chunk)) = stream.next().await {
            hasher.update(&chunk);
            file.write(&chunk).await.unwrap();
        }
    }
    let digest = hasher.finalize();

    Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/manifests/{}", repository_name, reference),
        )
        .header("Docker-Content-Digest", format!("sha256:{:x}", digest))
        .body(Body::empty())
        .unwrap()
}
