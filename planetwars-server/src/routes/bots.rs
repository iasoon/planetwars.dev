use axum::extract::{Multipart, Path, RawBody};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{self, PathBuf};

use crate::db::bots::{self, CodeBundle};
use crate::db::users::User;
use crate::DatabaseConnection;
use bots::Bot;

// TODO: make this a parameter
const BOTS_DIR: &str = "./data/bots";

#[derive(Serialize, Deserialize, Debug)]
pub struct BotParams {
    name: String,
}

pub async fn create_bot(
    conn: DatabaseConnection,
    user: User,
    params: Json<BotParams>,
) -> (StatusCode, Json<Bot>) {
    let bot_params = bots::NewBot {
        owner_id: user.id,
        name: &params.name,
    };
    let bot = bots::create_bot(&bot_params, &conn).unwrap();
    (StatusCode::CREATED, Json(bot))
}

// TODO: handle errors
pub async fn get_bot(conn: DatabaseConnection, Path(bot_id): Path<i32>) -> impl IntoResponse {
    bots::find_bot(bot_id, &conn)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn get_my_bots(
    conn: DatabaseConnection,
    user: User,
) -> Result<Json<Vec<Bot>>, StatusCode> {
    bots::find_bots_by_owner(user.id, &conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TODO: currently this only implements the happy flow
pub async fn upload_code_multipart(
    conn: DatabaseConnection,
    user: User,
    Path(bot_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<CodeBundle>, StatusCode> {
    let bots_dir = PathBuf::from(BOTS_DIR);

    let bot = bots::find_bot(bot_id, &conn).map_err(|_| StatusCode::NOT_FOUND)?;

    if user.id != bot.owner_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let data = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // TODO: this random path might be a bit redundant
    let folder_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    zip::ZipArchive::new(Cursor::new(data))
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .extract(bots_dir.join(&folder_name))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let bundle = bots::NewCodeBundle {
        bot_id: bot.id,
        path: &folder_name,
    };
    let code_bundle =
        bots::create_code_bundle(&bundle, &conn).expect("Failed to create code bundle");

    Ok(Json(code_bundle))
}
