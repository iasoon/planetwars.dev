use rand::Rng;
use rocket::data::ToByteUnit;
use rocket::fs::TempFile;
use rocket::Data;
use rocket::{response::status, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;

use crate::DbConn;

use crate::db::bots::{self, CodeBundle};
use crate::db::users::User;
use bots::Bot;

#[derive(Serialize, Deserialize, Debug)]
pub struct BotParams {
    name: String,
}

// TODO: handle errors
#[post("/bots", data = "<params>")]
pub async fn create_bot(
    db_conn: DbConn,
    user: User,
    params: Json<BotParams>,
) -> status::Created<Json<Bot>> {
    db_conn
        .run(move |conn| {
            let bot_params = bots::NewBot {
                owner_id: user.id,
                name: &params.name,
            };
            let bot = bots::create_bot(&bot_params, conn).unwrap();
            let bot_url = uri!(get_bot(bot.id)).to_string();
            status::Created::new(bot_url).body(Json(bot))
        })
        .await
}

// TODO: handle errors
#[get("/bots/<bot_id>")]
pub async fn get_bot(db_conn: DbConn, bot_id: i32) -> Json<Bot> {
    db_conn
        .run(move |conn| {
            let bot = bots::find_bot(bot_id, conn).unwrap();
            Json(bot)
        })
        .await
}

// TODO: proper error handling
#[post("/bots/<bot_id>/upload", data = "<data>")]
pub async fn upload_bot_code(
    db_conn: DbConn,
    user: User,
    bot_id: i32,
    data: Data<'_>,
) -> status::Created<Json<CodeBundle>> {
    // TODO: put in config somewhere
    let data_path = "./data/bots";

    let bot = db_conn
        .run(move |conn| bots::find_bot(bot_id, conn))
        .await
        .expect("Bot not found");

    assert_eq!(user.id, bot.owner_id);

    // generate a random filename
    let token: [u8; 16] = rand::thread_rng().gen();
    let name = base64::encode(&token);

    let path = Path::new(data_path).join(name);
    let capped_buf = data.open(10usize.megabytes()).into_bytes().await.unwrap();
    assert!(capped_buf.is_complete());
    let buf = capped_buf.into_inner();

    zip::ZipArchive::new(Cursor::new(buf))
        .unwrap()
        .extract(&path)
        .unwrap();

    let code_bundle = db_conn
        .run(move |conn| {
            let bundle = bots::NewCodeBundle {
                bot_id: bot.id,
                path: path.to_str().unwrap(),
            };
            bots::create_code_bundle(&bundle, conn).expect("Failed to create code bundle")
        })
        .await;

    // TODO: proper location
    status::Created::new("").body(Json(code_bundle))
}
