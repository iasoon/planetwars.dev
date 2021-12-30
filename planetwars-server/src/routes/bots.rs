use axum::extract::{Path, RawBody};
use axum::http::StatusCode;
use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path;

use crate::db::bots::{self, CodeBundle};
use crate::db::users::User;
use crate::DatabaseConnection;
use bots::Bot;

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
pub async fn get_bot(conn: DatabaseConnection, Path(bot_id): Path<i32>) -> Json<Bot> {
    let bot = bots::find_bot(bot_id, &conn).unwrap();
    Json(bot)
}

// TODO: proper error handling
pub async fn upload_bot_code(
    conn: DatabaseConnection,
    user: User,
    Path(bot_id): Path<i32>,
    RawBody(body): RawBody,
) -> (StatusCode, Json<CodeBundle>) {
    // TODO: put in config somewhere
    let data_path = "./data/bots";

    let bot = bots::find_bot(bot_id, &conn).expect("Bot not found");

    assert_eq!(user.id, bot.owner_id);

    // generate a random filename
    let token: [u8; 16] = rand::thread_rng().gen();
    let name = base64::encode(&token);

    let path = path::Path::new(data_path).join(name);
    // let capped_buf = data.open(10usize.megabytes()).into_bytes().await.unwrap();
    // assert!(capped_buf.is_complete());
    // let buf = capped_buf.into_inner();
    let buf = hyper::body::to_bytes(body).await.unwrap();

    zip::ZipArchive::new(Cursor::new(buf))
        .unwrap()
        .extract(&path)
        .unwrap();

    let bundle = bots::NewCodeBundle {
        bot_id: bot.id,
        path: path.to_str().unwrap(),
    };
    let code_bundle =
        bots::create_code_bundle(&bundle, &conn).expect("Failed to create code bundle");

    (StatusCode::CREATED, Json(code_bundle))
}
