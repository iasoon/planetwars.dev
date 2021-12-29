#![feature(async_closure)]
extern crate mozaic4_backend;
extern crate zip;

use rocket::http::{ContentType, Status};

mod util;
use mozaic4_backend::db::{bots, sessions, users};
use mozaic4_backend::DbConn;
use sessions::Session;
use users::{Credentials, User};
use util::{run_test, BearerAuth};

async fn user_with_session(conn: &DbConn) -> (User, Session) {
    conn.run(|conn| {
        let credentials = Credentials {
            username: "piepkonijn",
            password: "geheim123",
        };
        let user = users::create_user(&credentials, conn).unwrap();
        let session = sessions::create_session(&user, conn);
        (user, session)
    })
    .await
}

#[rocket::async_test]
async fn test_bot_create() {
    run_test(async move |client, conn| {
        let (user, session) = user_with_session(&conn).await;

        let response = client
            .post("/bots")
            .header(BearerAuth::new(session.token.clone()))
            .header(ContentType::JSON)
            .body(
                r#"{
                "name": "testbot"
            }"#,
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Created);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let resp_text = response.into_string().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&resp_text).unwrap();
        assert_eq!(json["name"], "testbot");
        assert_eq!(json["owner_id"], user.id);
    })
    .await
}

// create an example zipfile for bot upload
fn create_zip() -> std::io::Result<Vec<u8>> {
    use std::io::Write;
    use zip::write::FileOptions;

    let cursor = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);

    zip.start_file("test.txt", FileOptions::default())?;
    zip.write_all(b"sup brudi")?;
    let buf = zip.finish()?;
    Ok(buf.into_inner())
}

#[rocket::async_test]
async fn test_bot_upload() {
    run_test(async move |client, conn| {
        let (user, session) = user_with_session(&conn).await;

        let owner_id = user.id;
        let bot = conn
            .run(move |conn| {
                let new_bot = bots::NewBot {
                    name: "testbot",
                    owner_id: owner_id,
                };
                bots::create_bot(&new_bot, conn).unwrap()
            })
            .await;

        let zip_file = create_zip().unwrap();

        let response = client
            .post(format!("/bots/{}/upload", bot.id))
            .header(BearerAuth::new(session.token.clone()))
            .header(ContentType::JSON)
            .body(zip_file)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Created);
    })
    .await
}
