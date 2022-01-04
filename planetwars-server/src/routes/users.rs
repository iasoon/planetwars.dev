use crate::db::users::{Credentials, User};
use crate::db::{sessions, users};
use crate::DatabaseConnection;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::response::{Headers, IntoResponse, Response};
use axum::{async_trait, Json};
use serde::{Deserialize, Serialize};

type AuthorizationHeader = TypedHeader<Authorization<Bearer>>;

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let conn = DatabaseConnection::from_request(req).await?;

        let TypedHeader(Authorization(bearer)) = AuthorizationHeader::from_request(req)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "".to_string()))?;

        let (_session, user) = sessions::find_user_by_session(bearer.token(), &conn)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "".to_string()))?;

        Ok(user)
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
            user_id: user.id,
            username: user.username,
        }
    }
}

#[derive(Deserialize)]
pub struct RegistrationParams {
    pub username: String,
    pub password: String,
}

pub async fn register(
    conn: DatabaseConnection,
    params: Json<RegistrationParams>,
) -> Json<UserData> {
    let credentials = Credentials {
        username: &params.username,
        password: &params.password,
    };
    let user = users::create_user(&credentials, &conn).unwrap();
    Json(user.into())
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

pub async fn login(conn: DatabaseConnection, params: Json<LoginParams>) -> Response {
    let credentials = Credentials {
        username: &params.username,
        password: &params.password,
    };
    // TODO: handle failures
    let authenticated = users::authenticate_user(&credentials, &conn);

    match authenticated {
        None => StatusCode::FORBIDDEN.into_response(),
        Some(user) => {
            let session = sessions::create_session(&user, &conn);
            let user_data: UserData = user.into();
            let headers = Headers(vec![("Token", &session.token)]);

            (headers, Json(user_data)).into_response()
        }
    }
}

pub async fn current_user(user: User) -> Json<UserData> {
    Json(user.into())
}
