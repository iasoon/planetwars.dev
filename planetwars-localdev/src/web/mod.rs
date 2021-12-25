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
    net::SocketAddr,
    path::{self, PathBuf},
    sync::Arc,
};

struct State {
    project_root: PathBuf,
}

impl State {
    fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }
}

pub async fn run(project_root: PathBuf) {
    let shared_state = Arc::new(State::new(project_root));

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
struct Match {
    name: String,
}

async fn list_matches(Extension(state): Extension<Arc<State>>) -> Json<Vec<Match>> {
    let matches = state
        .project_root
        .join("matches")
        .read_dir()
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            extract_match_name(entry).map(|name| Match { name })
        })
        .collect::<Vec<_>>();
    Json(matches)
}

// extracts 'filename' if the entry matches'$filename.log'.
fn extract_match_name(entry: std::fs::DirEntry) -> Option<String> {
    let file_name = entry.file_name();
    let path = path::Path::new(&file_name);
    if path.extension() == Some("log".as_ref()) {
        path.file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    } else {
        None
    }
}

async fn get_match(Extension(state): Extension<Arc<State>>, Path(id): Path<String>) -> String {
    let mut match_path = state.project_root.join("matches").join(id);
    match_path.set_extension("log");
    std::fs::read_to_string(match_path).unwrap()
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
