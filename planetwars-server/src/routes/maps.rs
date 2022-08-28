use crate::{db, DatabaseConnection};
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiMap {
    pub name: String,
}

pub async fn list_maps(conn: DatabaseConnection) -> Result<Json<Vec<ApiMap>>, StatusCode> {
    let maps = db::maps::list_maps(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let api_maps = maps
        .into_iter()
        .map(|map| ApiMap { name: map.name })
        .collect();
    Ok(Json(api_maps))
}
