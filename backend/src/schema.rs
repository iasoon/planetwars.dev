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

joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(sessions, users,);
