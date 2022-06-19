use axum::body::{Body, Bytes, StreamBody};
use axum::extract::{BodyStream, FromRequest, Path, Query, RequestParts, TypedHeader};
use axum::handler::Handler;
use axum::headers::authorization::Basic;
use axum::headers::Authorization;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, head, post, put};
use axum::{async_trait, Router};
use hyper::StatusCode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::util::gen_alphanumeric;

const REGISTRY_PATH: &'static str = "./data/registry";
pub fn registry_service() -> Router {
    Router::new()
        // The docker API requires this trailing slash
        .nest("/v2/", registry_api_v2())
        .fallback(fallback.into_service())
}

fn registry_api_v2() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/:name/blobs/:digest", head(blob_check).get(get_blob))
        .route("/:name/blobs/uploads/", post(blob_upload))
        .route(
            "/:name/blobs/uploads/:uuid",
            put(put_handler).patch(handle_upload),
        )
        .route(
            "/:name/manifests/:reference",
            get(get_manifest).put(put_manifest),
        )
}

async fn fallback(request: axum::http::Request<Body>) -> impl IntoResponse {
    // for debugging
    println!("no route for {} {}", request.method(), request.uri());
    StatusCode::NOT_FOUND
}

type AuthorizationHeader = TypedHeader<Authorization<Basic>>;

struct RegistryAuth;

#[async_trait]
impl<B> FromRequest<B> for RegistryAuth
where
    B: Send,
{
    type Rejection = Response<axum::body::Full<Bytes>>;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(_basic)) =
            AuthorizationHeader::from_request(req).await.map_err(|_| {
                let err = RegistryErrors {
                    errors: vec![RegistryError {
                        code: "UNAUTHORIZED".to_string(),
                        message: "please log in".to_string(),
                        detail: serde_json::Value::Null,
                    }],
                };
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Docker-Distribution-API-Version", "registry/2.0")
                    .header("WWW-Authenticate", "Basic")
                    .body(axum::body::Full::from(serde_json::to_vec(&err).unwrap()))
                    .unwrap()
            })?;

        Ok(RegistryAuth)
    }
}

async fn root_handler(_auth: RegistryAuth) -> impl IntoResponse {
    // root should return 200 OK to confirm api compliance
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

async fn get_blob(
    Path((_repository_name, raw_digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let digest = raw_digest.strip_prefix("sha256:").unwrap();
    let blob_path = PathBuf::from(REGISTRY_PATH).join("sha256").join(&digest);
    if !blob_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }
    let file = tokio::fs::File::open(&blob_path).await.unwrap();
    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream);
    Ok(stream_body)
}

async fn blob_upload(Path(repository_name): Path<String>) -> impl IntoResponse {
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

async fn get_manifest(
    Path((repository_name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let manifest_path = PathBuf::from(REGISTRY_PATH)
        .join("manifests")
        .join(&repository_name)
        .join(&reference)
        .with_extension("json");
    let data = tokio::fs::read(&manifest_path).await.unwrap();

    let manifest: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&data).unwrap();
    let media_type = manifest.get("mediaType").unwrap().as_str().unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", media_type)
        .body(axum::body::Full::from(data))
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
    let manifest_path = repository_dir.join(&reference).with_extension("json");
    {
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
    // TODO: store content-adressable manifests separately
    let content_digest = format!("sha256:{:x}", digest);
    let digest_path = repository_dir.join(&content_digest).with_extension("json");
    tokio::fs::copy(manifest_path, digest_path).await.unwrap();

    Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/manifests/{}", repository_name, reference),
        )
        .header("Docker-Content-Digest", content_digest)
        .body(Body::empty())
        .unwrap()
}
