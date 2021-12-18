#![feature(async_closure)]
extern crate mozaic4_backend;

use rocket::http::{ContentType, Header, Status};

mod util;
use util::run_test;

pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl<'a> Into<Header<'a>> for BearerAuth {
    fn into(self) -> Header<'a> {
        Header::new("Authorization", format!("Bearer {}", self.token))
    }
}

#[rocket::async_test]
async fn test_registration() {
    run_test(async move |client, _conn| {
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
            .header(BearerAuth::new(token))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let resp = response.into_string().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(json["username"], "piepkonijn");
    }).await
}

#[rocket::async_test]
async fn test_reject_invalid_credentials() {
    run_test(async move |client, _conn| {
        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"username": "piepkonijn", "password": "letmeinplease"}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);
        // assert_eq!(response.content_type(), Some(ContentType::JSON));
    }).await
}
