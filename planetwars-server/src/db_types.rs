use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[DieselTypePath = "crate::schema::sql_types::MatchState"]

pub enum MatchState {
    Playing,
    Finished,
}
