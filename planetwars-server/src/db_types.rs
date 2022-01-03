use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[DieselType = "Match_state"]

pub enum MatchState {
    Playing,
    Finished,
}
