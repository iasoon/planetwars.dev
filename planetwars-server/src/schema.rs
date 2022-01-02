table! {
    bots (id) {
        id -> Int4,
        owner_id -> Int4,
        name -> Text,
    }
}

table! {
    code_bundles (id) {
        id -> Int4,
        bot_id -> Int4,
        path -> Text,
        created_at -> Timestamp,
    }
}

table! {
    match_players (match_id, player_id) {
        match_id -> Int4,
        bot_id -> Int4,
        player_id -> Int4,
    }
}

table! {
    matches (id) {
        id -> Int4,
        log_path -> Text,
        created_at -> Timestamp,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        token -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password_salt -> Bytea,
        password_hash -> Bytea,
    }
}

joinable!(bots -> users (owner_id));
joinable!(code_bundles -> bots (bot_id));
joinable!(match_players -> bots (bot_id));
joinable!(match_players -> matches (match_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(bots, code_bundles, match_players, matches, sessions, users,);
