// TODO: this module is functional, but it needs a good refactor for proper error handling.

use axum::body::{Body, StreamBody};
use axum::extract::{BodyStream, FromRequest, Path, Query, RequestParts, TypedHeader};
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

use crate::db::bots::NewBotVersion;
use crate::util::gen_alphanumeric;
use crate::{db, DatabaseConnection};

use crate::db::users::{authenticate_user, Credentials, User};

// TODO: put this in a config file
const REGISTRY_PATH: &str = "./data/registry";

pub fn registry_service() -> Router {
    Router::new()
        // The docker API requires this trailing slash
        .nest("/v2/", registry_api_v2())
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

const ADMIN_USERNAME: &str = "admin";
// TODO: put this in some configuration
const ADMIN_PASSWORD: &str = "supersecretpassword";

type AuthorizationHeader = TypedHeader<Authorization<Basic>>;

enum RegistryAuth {
    User(User),
    Admin,
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
        let TypedHeader(Authorization(basic)) = AuthorizationHeader::from_request(req)
            .await
            .map_err(|_| RegistryAuthError::NoAuthHeader)?;

        // TODO: Into<Credentials> would be nice
        let credentials = Credentials {
            username: basic.username(),
            password: basic.password(),
        };

        if credentials.username == ADMIN_USERNAME {
            if credentials.password == ADMIN_PASSWORD {
                Ok(RegistryAuth::Admin)
            } else {
                Err(RegistryAuthError::InvalidCredentials)
            }
        } else {
            let db_conn = DatabaseConnection::from_request(req).await.unwrap();
            let user = authenticate_user(&credentials, &db_conn)
                .ok_or(RegistryAuthError::InvalidCredentials)?;

            Ok(RegistryAuth::User(user))
        }
    }
}

// Since async file io just calls spawn_blocking internally, it does not really make sense
// to make this an async function
fn file_sha256_digest(path: &std::path::Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let _n = std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get the index of the last byte in a file
async fn last_byte_pos(file: &tokio::fs::File) -> std::io::Result<u64> {
    let n_bytes = file.metadata().await?.len();
    let pos = if n_bytes == 0 { 0 } else { n_bytes - 1 };
    Ok(pos)
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
        let metadata = std::fs::metadata(&blob_path).unwrap();
        Ok((StatusCode::OK, [("Content-Length", metadata.len())]))
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

    // TODO: support content range header in request
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
        file.write_all(&chunk).await.unwrap();
    }

    let last_byte = last_byte_pos(&file).await.unwrap();

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(
            "Location",
            format!("/v2/{}/blobs/uploads/{}", repository_name, uuid),
        )
        .header("Docker-Upload-UUID", uuid)
        // range indicating current progress of the upload
        .header("Range", format!("0-{}", last_byte))
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

    let upload_path = PathBuf::from(REGISTRY_PATH).join("uploads").join(&uuid);
    let mut file = tokio::fs::OpenOptions::new()
        .read(false)
        .write(true)
        .append(true)
        .create(false)
        .open(&upload_path)
        .await
        .unwrap();

    let range_begin = last_byte_pos(&file).await.unwrap();
    while let Some(Ok(chunk)) = stream.next().await {
        file.write_all(&chunk).await.unwrap();
    }
    file.flush().await.unwrap();
    let range_end = last_byte_pos(&file).await.unwrap();

    let expected_digest = params.digest.strip_prefix("sha256:").unwrap();
    let digest = file_sha256_digest(&upload_path).unwrap();
    if digest != expected_digest {
        // TODO: return a docker error body
        return Err(StatusCode::BAD_REQUEST);
    }

    let target_path = PathBuf::from(REGISTRY_PATH).join("sha256").join(&digest);
    tokio::fs::rename(&upload_path, &target_path).await.unwrap();

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(
            "Location",
            format!("/v2/{}/blobs/{}", repository_name, digest),
        )
        .header("Docker-Upload-UUID", uuid)
        // content range for bytes that were in the body of this request
        .header("Content-Range", format!("{}-{}", range_begin, range_end))
        .header("Docker-Content-Digest", params.digest)
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
    let bot = check_access(&repository_name, &auth, &db_conn)?;

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
            file.write_all(&chunk).await.unwrap();
        }
    }
    let digest = hasher.finalize();
    // TODO: store content-adressable manifests separately
    let content_digest = format!("sha256:{:x}", digest);
    let digest_path = repository_dir.join(&content_digest).with_extension("json");
    tokio::fs::copy(manifest_path, digest_path).await.unwrap();

    // Register the new image as a bot version
    // TODO: how should tags be handled?
    let new_version = NewBotVersion {
        bot_id: Some(bot.id),
        code_bundle_path: None,
        container_digest: Some(&content_digest),
    };
    db::bots::create_bot_version(&new_version, &db_conn).expect("could not save bot version");

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

/// Ensure that the accessed repository exists
/// and the user is allowed to access it.
/// Returns the associated bot.
fn check_access(
    repository_name: &str,
    auth: &RegistryAuth,
    db_conn: &DatabaseConnection,
) -> Result<db::bots::Bot, StatusCode> {
    use diesel::OptionalExtension;

    // TODO: it would be nice to provide the found repository
    // to the route handlers
    let bot = db::bots::find_bot_by_name(repository_name, db_conn)
        .optional()
        .expect("could not run query")
        .ok_or(StatusCode::NOT_FOUND)?;

    match &auth {
        RegistryAuth::Admin => Ok(bot),
        RegistryAuth::User(user) => {
            if bot.owner_id == Some(user.id) {
                Ok(bot)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}
