use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[DieselType = "Match_state"]

pub enum MatchState {
    Playing,
    Finished,
}
