use crate::db::{sessions, users};
use crate::{
    db::users::{Credentials, User},
    DbConn,
};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[derive(Debug)]
pub enum AuthTokenError {
    BadCount,
    Missing,
    Invalid,
}

// TODO: error handling and proper lifetimes
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        let token = match keys.len() {
            0 => return Outcome::Failure((Status::BadRequest, AuthTokenError::Missing)),
            1 => keys[0].to_string(),
            _ => return Outcome::Failure((Status::BadRequest, AuthTokenError::BadCount)),
        };
        let db = request.guard::<DbConn>().await.unwrap();
        let (_session, user) = db
            .run(move |conn| sessions::find_user_by_session(&token, conn))
            .await
            .unwrap();
        Outcome::Success(user)
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserData {
    pub user_id: i32,
    pub username: String,
}

impl From<User> for UserData {
    fn from(user: User) -> Self {
        UserData {
            user_id: user.user_id,
            username: user.username,
        }
    }
}

#[derive(Deserialize)]
pub struct RegistrationParams {
    pub username: String,
    pub password: String,
}

#[post("/register", data = "<params>")]
pub async fn register(db_conn: DbConn, params: Json<RegistrationParams>) -> Json<UserData> {
    db_conn
        .run(move |conn| {
            let credentials = Credentials {
                username: &params.username,
                password: &params.password,
            };
            let user = users::create_user(&credentials, conn).unwrap();
            Json(user.into())
        })
        .await
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

#[post("/login", data = "<params>")]
pub async fn login(db_conn: DbConn, params: Json<LoginParams>) -> String {
    db_conn
        .run(move |conn| {
            let credentials = Credentials {
                username: &params.username,
                password: &params.password,
            };
            // TODO: handle failures
            let user = users::authenticate_user(&credentials, conn).unwrap();
            let session = sessions::create_session(&user, conn);
            return session.token;
        })
        .await
}

#[get("/users/me")]
pub async fn current_user(user: User) -> Json<UserData> {
    Json(user.into())
}
