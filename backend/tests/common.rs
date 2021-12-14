extern crate mozaic4_backend;

use diesel;
use diesel::prelude::*;
use mozaic4_backend::DbConn;
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;

// We use a lock to synchronize between tests so DB operations don't collide.
// For now. In the future, we'll have a nice way to run each test in a DB
// transaction so we can regain concurrency.
static DB_LOCK: parking_lot::Mutex<()> = parking_lot::const_mutex(());

async fn reset_db(db: &DbConn) {
    db.run(|conn| {
        diesel::sql_query("TRUNCATE TABLE users, sessions")
            .execute(conn)
            .expect("drop all tables");
    })
    .await
}

macro_rules! run_test {
    (|$client:ident, $conn:ident| $block:expr) => {{
        let _lock = DB_LOCK.lock();

        rocket::async_test(async move {
            let $client = Client::tracked(mozaic4_backend::rocket())
                .await
                .expect("Rocket client");
            let db = mozaic4_backend::DbConn::get_one($client.rocket()).await;
            let $conn = db.expect("failed to get database connection for testing");
            reset_db(&$conn).await;

            $block
        })
    }};
}

#[test]
fn test_registration() {
    run_test!(|client, _conn| {
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{"username": "piepkonijn", "password": "geheim123"}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"username": "piepkonijn", "password": "geheim123"}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let token = response.into_string().await.unwrap();

        let response = client
            .get("/users/me")
            .header(Header::new("Authorization", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let resp = response.into_string().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(json["username"], "piepkonijn");
    });
}
