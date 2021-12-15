use crate::db::{sessions, users};
use crate::{
    db::users::{Credentials, User},
    DbConn,
};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;

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
        let auth_header = match keys.len() {
            0 => return Outcome::Failure((Status::BadRequest, AuthTokenError::Missing)),
            1 => keys[0],
            _ => return Outcome::Failure((Status::BadRequest, AuthTokenError::BadCount)),
        };

        let token = match auth_header.strip_prefix("Bearer ") {
            Some(token) => token.to_string(),
            None => return Outcome::Failure((Status::BadRequest, AuthTokenError::Invalid)),
        };

        let db = request.guard::<DbConn>().await.unwrap();
        let res = db
            .run(move |conn| sessions::find_user_by_session(&token, conn))
            .await;
        match res {
            Ok((_session, user)) => Outcome::Success(user),
            Err(_) => Outcome::Failure((Status::Unauthorized, AuthTokenError::Invalid)),
        }
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
pub async fn login(
    db_conn: DbConn,
    params: Json<LoginParams>,
) -> Result<String, status::Forbidden<&'static str>> {
    db_conn
        .run(move |conn| {
            let credentials = Credentials {
                username: &params.username,
                password: &params.password,
            };
            // TODO: handle failures
            let authenticated = users::authenticate_user(&credentials, conn);

            match authenticated {
                None => Err(status::Forbidden(Some("invalid auth"))),
                Some(user) => {
                    let session = sessions::create_session(&user, conn);
                    Ok(session.token)
                }
            }
        })
        .await
}

#[get("/users/me")]
pub async fn current_user(user: User) -> Json<UserData> {
    Json(user.into())
}
