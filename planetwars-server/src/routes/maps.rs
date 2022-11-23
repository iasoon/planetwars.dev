use std::{collections::HashSet, fs::File, path::PathBuf, sync::Arc};

use crate::{db, DatabaseConnection, GlobalConfig};
use axum::{Extension, Json};
use diesel::OptionalExtension;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiMap {
    pub name: String,
}

fn map_into_api_map(map: db::maps::Map) -> ApiMap {
    ApiMap { name: map.name }
}

pub async fn list_maps(mut conn: DatabaseConnection) -> Result<Json<Vec<ApiMap>>, StatusCode> {
    let maps = db::maps::list_maps(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let api_maps = maps.into_iter().map(map_into_api_map).collect();
    Ok(Json(api_maps))
}

use planetwars_rules::config::Map as PlanetwarsMap;
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct CreateMapRequest {
    name: String,
    #[serde(flatten)]
    map: PlanetwarsMap,
}

pub async fn create_map(
    mut conn: DatabaseConnection,
    Extension(config): Extension<Arc<GlobalConfig>>,
    params: Json<CreateMapRequest>,
) -> Result<Json<ApiMap>, (StatusCode, String)> {
    match db::maps::find_map_by_name(&params.name, &mut conn).optional() {
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "internal error"
                })
                .to_string(),
            ))
        }
        Ok(Some(_)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "name taken",
                })
                .to_string(),
            ))
        }
        Ok(None) => {}
    };

    if let Err(error) = check_map_name(&params.name) {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": error,
            })
            .to_string(),
        ));
    }

    if let Err(error) = check_map(&params.map) {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({
                "error": error,
            })
            .to_string(),
        ));
    }

    let rel_map_path = format!("{}.json", &params.name);

    {
        let full_map_path = PathBuf::from(&config.maps_directory).join(&rel_map_path);
        let file = File::create(full_map_path).expect("failed to open file");
        serde_json::to_writer_pretty(file, &params.map).expect("failed to write map");
    }
    let map = db::maps::create_map(
        db::maps::NewMap {
            name: &params.name,
            file_path: &rel_map_path,
        },
        &mut conn,
    )
    .expect("failed to save map");

    Ok(Json(map_into_api_map(map)))
}

fn check_map(map: &PlanetwarsMap) -> Result<(), &str> {
    let unique_names: HashSet<String> = map.planets.iter().map(|p| p.name.clone()).collect();
    if unique_names.len() != map.planets.len() {
        return Err("planet names not unique");
    }
    let players: HashSet<usize> = map.planets.iter().filter_map(|p| p.owner).collect();

    if players != HashSet::from([1, 2]) {
        return Err("maps should have player 1 and 2");
    }

    Ok(())
}

// TODO: remove duplication (bot name, user name)
fn check_map_name(name: &str) -> Result<(), &str> {
    if !name
        .chars()
        .all(|c| !c.is_uppercase() && (c.is_ascii_alphanumeric() || c == '_' || c == '-'))
    {
        return Err("Only [a-z-_] are allowed in map names");
    }

    if name.len() < 3 || name.len() > 32 {
        return Err("map name should be between 3 and 32 characters");
    }

    Ok(())
}
