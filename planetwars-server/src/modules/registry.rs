use axum::body::{Body, StreamBody};
use axum::extract::{BodyStream, FromRequest, Path, Query, RequestParts, TypedHeader};
use axum::handler::Handler;
use axum::headers::authorization::Basic;
use axum::headers::Authorization;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, head, post, put};
use axum::{async_trait, Router};
use futures::StreamExt;
use hyper::StatusCode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::util::gen_alphanumeric;
use crate::{db, DatabaseConnection};

use crate::db::users::{authenticate_user, Credentials, User};

const REGISTRY_PATH: &str = "./data/registry";

pub fn registry_service() -> Router {
    Router::new()
        // The docker API requires this trailing slash
        .nest("/v2/", registry_api_v2())
        .fallback(fallback.into_service())
}

fn registry_api_v2() -> Router {
    Router::new()
        .route("/", get(get_root))
        .route(
            "/:name/manifests/:reference",
            get(get_manifest).put(put_manifest),
        )
        .route(
            "/:name/blobs/:digest",
            head(check_blob_exists).get(get_blob),
        )
        .route("/:name/blobs/uploads/", post(create_upload))
        .route(
            "/:name/blobs/uploads/:uuid",
            put(put_upload).patch(patch_upload),
        )
}

async fn fallback(request: axum::http::Request<Body>) -> impl IntoResponse {
    // for debugging
    println!("no route for {} {}", request.method(), request.uri());
    StatusCode::NOT_FOUND
}

type AuthorizationHeader = TypedHeader<Authorization<Basic>>;

enum RegistryAuth {
    User(User),
}

enum RegistryAuthError {
    NoAuthHeader,
    InvalidCredentials,
}

impl IntoResponse for RegistryAuthError {
    fn into_response(self) -> Response {
        // TODO: create enum for registry errors
        let err = RegistryErrors {
            errors: vec![RegistryError {
                code: "UNAUTHORIZED".to_string(),
                message: "please log in".to_string(),
                detail: serde_json::Value::Null,
            }],
        };

        (
            StatusCode::UNAUTHORIZED,
            [
                ("Docker-Distribution-API-Version", "registry/2.0"),
                ("WWW-Authenticate", "Basic"),
            ],
            serde_json::to_string(&err).unwrap(),
        )
            .into_response()
    }
}

#[async_trait]
impl<B> FromRequest<B> for RegistryAuth
where
    B: Send,
{
    type Rejection = RegistryAuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let db_conn = DatabaseConnection::from_request(req).await.unwrap();

        let TypedHeader(Authorization(basic)) = AuthorizationHeader::from_request(req)
            .await
            .map_err(|_| RegistryAuthError::NoAuthHeader)?;

        // TODO: Into<Credentials> would be nice
        let credentials = Credentials {
            username: basic.username(),
            password: basic.password(),
        };
        let user = authenticate_user(&credentials, &db_conn)
            .ok_or(RegistryAuthError::InvalidCredentials)?;

        Ok(RegistryAuth::User(user))
    }
}

async fn get_root(_auth: RegistryAuth) -> impl IntoResponse {
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

async fn check_blob_exists(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, raw_digest)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

    let digest = raw_digest.strip_prefix("sha256:").unwrap();
    let blob_path = PathBuf::from(REGISTRY_PATH).join("sha256").join(&digest);
    if blob_path.exists() {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_blob(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, raw_digest)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

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

async fn create_upload(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path(repository_name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

    let uuid = gen_alphanumeric(16);
    tokio::fs::File::create(PathBuf::from(REGISTRY_PATH).join("uploads").join(&uuid))
        .await
        .unwrap();

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(
            "Location",
            format!("/v2/{}/blobs/uploads/{}", repository_name, uuid),
        )
        .header("Docker-Upload-UUID", uuid)
        .header("Range", "bytes=0-0")
        .body(Body::empty())
        .unwrap())
}

async fn patch_upload(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, uuid)): Path<(String, String)>,
    mut stream: BodyStream,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

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

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(
            "Location",
            format!("/v2/{}/blobs/uploads/{}", repository_name, uuid),
        )
        .header("Docker-Upload-UUID", uuid)
        .header("Range", format!("0-{}", len))
        .body(Body::empty())
        .unwrap())
}

use serde::Deserialize;
#[derive(Deserialize)]
struct UploadParams {
    digest: String,
}

async fn put_upload(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, uuid)): Path<(String, String)>,
    Query(params): Query<UploadParams>,
    mut stream: BodyStream,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

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

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/blobs/{}", repository_name, digest),
        )
        .header("Docker-Upload-UUID", uuid)
        // .header("Range", format!("0-{}", len))
        .header("Docker-Content-Digest", digest)
        .body(Body::empty())
        .unwrap())
}

async fn get_manifest(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

    let manifest_path = PathBuf::from(REGISTRY_PATH)
        .join("manifests")
        .join(&repository_name)
        .join(&reference)
        .with_extension("json");
    let data = tokio::fs::read(&manifest_path).await.unwrap();

    let manifest: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&data).unwrap();
    let media_type = manifest.get("mediaType").unwrap().as_str().unwrap();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", media_type)
        .body(axum::body::Full::from(data))
        .unwrap())
}

async fn put_manifest(
    db_conn: DatabaseConnection,
    auth: RegistryAuth,
    Path((repository_name, reference)): Path<(String, String)>,
    mut stream: BodyStream,
) -> Result<impl IntoResponse, StatusCode> {
    check_access(&repository_name, &auth, &db_conn)?;

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

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/manifests/{}", repository_name, reference),
        )
        .header("Docker-Content-Digest", content_digest)
        .body(Body::empty())
        .unwrap())
}

fn check_access(
    repository_name: &str,
    auth: &RegistryAuth,
    db_conn: &DatabaseConnection,
) -> Result<(), StatusCode> {
    use diesel::OptionalExtension;

    let res = db::bots::find_bot_by_name(repository_name, db_conn)
        .optional()
        .expect("could not run query");

    match res {
        None => Ok(()), // name has not been claimed yet (TODO: verify its validity)
        Some(existing_bot) => {
            let RegistryAuth::User(user) = auth;
            if existing_bot.owner_id == Some(user.id) {
                Ok(())
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}
