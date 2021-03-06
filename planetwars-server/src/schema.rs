// This file is autogenerated by diesel
#![allow(unused_imports)]

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    bot_versions (id) {
        id -> Int4,
        bot_id -> Nullable<Int4>,
        code_bundle_path -> Nullable<Text>,
        created_at -> Timestamp,
        container_digest -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    bots (id) {
        id -> Int4,
        owner_id -> Nullable<Int4>,
        name -> Text,
        active_version -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    match_players (match_id, player_id) {
        match_id -> Int4,
        player_id -> Int4,
        bot_version_id -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    matches (id) {
        id -> Int4,
        state -> Match_state,
        log_path -> Text,
        created_at -> Timestamp,
        winner -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    ratings (bot_id) {
        bot_id -> Int4,
        rating -> Float8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        token -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password_salt -> Bytea,
        password_hash -> Bytea,
    }
}

joinable!(bots -> users (owner_id));
joinable!(match_players -> bot_versions (bot_version_id));
joinable!(match_players -> matches (match_id));
joinable!(ratings -> bots (bot_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    bot_versions,
    bots,
    match_players,
    matches,
    ratings,
    sessions,
    users,
);
