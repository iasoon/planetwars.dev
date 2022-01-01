use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MatchParams {
    // Just bot ids for now
    players: Vec<i32>,
}

pub async fn play_match(params: Json<MatchParams>) {
    println!("start match: {:#?}", params);
}
