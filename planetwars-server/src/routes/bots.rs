use axum::extract::{Multipart, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{body, Extension, Json};
use diesel::OptionalExtension;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, value::Value as JsonValue};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror;

use crate::db;
use crate::db::bots::{self, BotVersion};
use crate::db::ratings::{self, RankedBot};
use crate::db::users::User;
use crate::modules::bots::save_code_string;
use crate::{DatabaseConnection, GlobalConfig};
use bots::Bot;

use super::users::UserData;

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveBotParams {
    pub bot_name: String,
    pub code: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveBotError {
    #[error("database error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("validation failed")]
    ValidationFailed(Vec<&'static str>),
    #[error("bot name already exists")]
    BotNameTaken,
}

impl IntoResponse for SaveBotError {
    fn into_response(self) -> Response {
        let (status, value) = match self {
            SaveBotError::BotNameTaken => (
                StatusCode::FORBIDDEN,
                json!({ "error": {
                    "type": "bot_name_taken",
                } }),
            ),
            SaveBotError::DatabaseError(_e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": {
                    "type": "internal_server_error",
                } }),
            ),
            SaveBotError::ValidationFailed(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": {
                    "type": "validation_failed",
                    "validation_errors": errors,
                } }),
            ),
        };

        let encoded = serde_json::to_vec(&value).expect("could not encode response value");

        Response::builder()
            .status(status)
            .body(body::boxed(body::Full::from(encoded)))
            .expect("could not build response")
    }
}

pub fn validate_bot_name(bot_name: &str) -> Result<(), SaveBotError> {
    let mut errors = Vec::new();

    if bot_name.len() < 3 {
        errors.push("bot name must be at least 3 characters long");
    }

    if bot_name.len() > 32 {
        errors.push("bot name must be at most 32 characters long");
    }

    if !bot_name
        .chars()
        .all(|c| !c.is_uppercase() && (c.is_ascii_alphanumeric() || c == '_' || c == '-'))
    {
        errors.push("only lowercase alphanumeric characters, underscores, and dashes are allowed in bot names");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(SaveBotError::ValidationFailed(errors))
    }
}

pub async fn save_bot(
    Json(params): Json<SaveBotParams>,
    user: User,
    conn: DatabaseConnection,
    Extension(config): Extension<Arc<GlobalConfig>>,
) -> Result<Json<Bot>, SaveBotError> {
    let res = bots::find_bot_by_name(&params.bot_name, &conn)
        .optional()
        .expect("could not run query");

    let bot = match res {
        Some(existing_bot) => {
            if existing_bot.owner_id == Some(user.id) {
                existing_bot
            } else {
                return Err(SaveBotError::BotNameTaken);
            }
        }
        None => {
            validate_bot_name(&params.bot_name)?;
            let new_bot = bots::NewBot {
                owner_id: Some(user.id),
                name: &params.bot_name,
            };

            bots::create_bot(&new_bot, &conn).expect("could not create bot")
        }
    };
    let _code_bundle = save_code_string(&params.code, Some(bot.id), &conn, &config)
        .expect("failed to save code bundle");
    Ok(Json(bot))
}

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
        owner_id: Some(user.id),
        name: &params.name,
    };
    let bot = bots::create_bot(&bot_params, &conn).unwrap();
    (StatusCode::CREATED, Json(bot))
}

// TODO: handle errors
pub async fn get_bot(
    conn: DatabaseConnection,
    Path(bot_name): Path<String>,
) -> Result<Json<JsonValue>, StatusCode> {
    let bot = db::bots::find_bot_by_name(&bot_name, &conn).map_err(|_| StatusCode::NOT_FOUND)?;
    let owner: Option<UserData> = match bot.owner_id {
        Some(user_id) => {
            let user = db::users::find_user(user_id, &conn)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Some(user.into())
        }
        None => None,
    };
    let versions =
        bots::find_bot_versions(bot.id, &conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({
        "bot": bot,
        "owner": owner,
        "versions": versions,
    })))
}

pub async fn get_user_bots(
    conn: DatabaseConnection,
    Path(user_name): Path<String>,
) -> Result<Json<Vec<Bot>>, StatusCode> {
    let user =
        db::users::find_user_by_name(&user_name, &conn).map_err(|_| StatusCode::NOT_FOUND)?;
    db::bots::find_bots_by_owner(user.id, &conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// List all active bots
pub async fn list_bots(conn: DatabaseConnection) -> Result<Json<Vec<Bot>>, StatusCode> {
    bots::find_active_bots(&conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_ranking(conn: DatabaseConnection) -> Result<Json<Vec<RankedBot>>, StatusCode> {
    ratings::get_bot_ranking(&conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TODO: currently this only implements the happy flow
pub async fn upload_code_multipart(
    conn: DatabaseConnection,
    user: User,
    Path(bot_name): Path<String>,
    mut multipart: Multipart,
    Extension(config): Extension<Arc<GlobalConfig>>,
) -> Result<Json<BotVersion>, StatusCode> {
    let bots_dir = PathBuf::from(&config.bots_directory);

    let bot = bots::find_bot_by_name(&bot_name, &conn).map_err(|_| StatusCode::NOT_FOUND)?;

    if Some(user.id) != bot.owner_id {
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

    let bot_version = bots::NewBotVersion {
        bot_id: Some(bot.id),
        code_bundle_path: Some(&folder_name),
        container_digest: None,
    };
    let code_bundle =
        bots::create_bot_version(&bot_version, &conn).expect("Failed to create code bundle");

    Ok(Json(code_bundle))
}
