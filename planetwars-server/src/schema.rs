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
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(bots, code_bundles, sessions, users,);
