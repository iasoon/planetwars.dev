use axum::{
    body::{boxed, Full},
    extract::{Extension, Path},
    handler::Handler,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, Router},
    AddExtensionLayer, Json,
};
use mime_guess;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, BufRead},
    net::SocketAddr,
    path::{self, PathBuf},
    sync::Arc,
};

use crate::match_runner::MatchMeta;

struct State {
    workspace_root: PathBuf,
}

impl State {
    fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

pub async fn run(workspace_root: PathBuf) {
    let shared_state = Arc::new(State::new(workspace_root));

    // build our application with a route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/matches", get(list_matches))
        .route("/api/matches/:match_id", get(get_match))
        .fallback(static_handler.into_service())
        .layer(AddExtensionLayer::new(shared_state));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("serving at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct MatchData {
    name: String,
    #[serde(flatten)]
    meta: MatchMeta,
}

async fn list_matches(Extension(state): Extension<Arc<State>>) -> Json<Vec<MatchData>> {
    let matches = state
        .workspace_root
        .join("matches")
        .read_dir()
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            get_match_data(&entry).ok()
        })
        .collect::<Vec<_>>();
    Json(matches)
}

// extracts 'filename' if the entry matches'$filename.log'.
fn get_match_data(entry: &fs::DirEntry) -> io::Result<MatchData> {
    let file_name = entry.file_name();
    let path = path::Path::new(&file_name);

    let name = get_match_name(&path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid match name"))?;

    let meta = read_match_meta(&entry.path())?;

    Ok(MatchData { name, meta })
}

fn get_match_name(path: &path::Path) -> Option<String> {
    if path.extension() != Some("log".as_ref()) {
        return None;
    }

    path.file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

fn read_match_meta(path: &path::Path) -> io::Result<MatchMeta> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let meta: MatchMeta = serde_json::from_str(&line)?;
    Ok(meta)
}

async fn get_match(Extension(state): Extension<Arc<State>>, Path(id): Path<String>) -> String {
    let mut match_path = state.workspace_root.join("matches").join(id);
    match_path.set_extension("log");
    fs::read_to_string(match_path).unwrap()
}

async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

// static_handler is a handler that serves static files from the
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}

#[derive(RustEmbed)]
#[folder = "../web/pw-frontend/dist/"]
struct Asset;
pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();
        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}
