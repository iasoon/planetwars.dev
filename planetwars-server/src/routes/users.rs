use crate::db::users::{Credentials, User};
use crate::db::{sessions, users};
use crate::DatabaseConnection;
use axum::extract::{FromRequest, RequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

const RESERVED_USERNAMES: &[&str] = &["admin", "system"];

type AuthorizationHeader = TypedHeader<Authorization<Bearer>>;

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let mut conn = DatabaseConnection::from_request(req).await?;

        let TypedHeader(Authorization(bearer)) = AuthorizationHeader::from_request(req)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "".to_string()))?;

        let (_session, user) = sessions::find_user_by_session(bearer.token(), &mut conn)
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

#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("database error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("validation failed")]
    ValidationFailed(Vec<String>),
}

impl RegistrationParams {
    fn validate(&self, conn: &mut DatabaseConnection) -> Result<(), RegistrationError> {
        let mut errors = Vec::new();

        // TODO: do we want to support cased usernames?
        // this could be done by allowing casing in names, but requiring case-insensitive uniqueness
        if !self
            .username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() && !c.is_uppercase())
        {
            errors.push("username must be alphanumeric and lowercase".to_string());
        }

        if self.username.len() < 3 {
            errors.push("username must be at least 3 characters".to_string());
        }

        if self.username.len() > 32 {
            errors.push("username must be at most 32 characters".to_string());
        }

        if self.password.len() < 8 {
            errors.push("password must be at least 8 characters".to_string());
        }

        if RESERVED_USERNAMES.contains(&self.username.as_str()) {
            errors.push("that username is not allowed".to_string());
        }

        if users::find_user_by_name(&self.username, conn).is_ok() {
            errors.push("username is already taken".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(RegistrationError::ValidationFailed(errors))
        }
    }
}

impl IntoResponse for RegistrationError {
    fn into_response(self) -> Response {
        let (status, json_body) = match self {
            RegistrationError::DatabaseError(_e) => {
                // TODO: create an error response struct
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": {
                            "type": "internal_server_error",
                        }
                    }),
                )
            }
            RegistrationError::ValidationFailed(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "error": {
                        "type": "validation_failed",
                        "validation_errors": errors,
                    }
                }),
            ),
        };

        (status, Json(json_body)).into_response()
    }
}

pub async fn register(
    mut conn: DatabaseConnection,
    params: Json<RegistrationParams>,
) -> Result<Json<UserData>, RegistrationError> {
    params.validate(&mut conn)?;

    let credentials = Credentials {
        username: &params.username,
        password: &params.password,
    };
    let user = users::create_user(&credentials, &mut conn)?;
    Ok(Json(user.into()))
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

pub async fn login(mut conn: DatabaseConnection, params: Json<LoginParams>) -> Response {
    let credentials = Credentials {
        username: &params.username,
        password: &params.password,
    };
    // TODO: handle failures
    let authenticated = users::authenticate_user(&credentials, &mut conn);

    match authenticated {
        None => StatusCode::FORBIDDEN.into_response(),
        Some(user) => {
            let session = sessions::create_session(&user, &mut conn);
            let user_data: UserData = user.into();
            let headers = [("Token", &session.token)];

            (StatusCode::OK, headers, Json(user_data)).into_response()
        }
    }
}

pub async fn current_user(user: User) -> Json<UserData> {
    Json(user.into())
}
