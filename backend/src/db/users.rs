use crate::{schema::users, DbConn};
use argon2;
use diesel::{prelude::*, PgConnection};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Credentials<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a [u8],
    pub password_salt: &'a [u8],
}

#[derive(Queryable, Debug)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub password_salt: Vec<u8>,
    pub password_hash: Vec<u8>,
}

// TODO: make this configurable somewhere
fn argon2_config() -> argon2::Config<'static> {
    argon2::Config {
        variant: argon2::Variant::Argon2i,
        version: argon2::Version::Version13,
        mem_cost: 4096,
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        // TODO: set a secret
        secret: &[],
        ad: &[],
        hash_length: 32,
    }
}

pub fn create_user(credentials: &Credentials, conn: &PgConnection) -> QueryResult<User> {
    let argon_config = argon2_config();

    let salt: [u8; 32] = rand::thread_rng().gen();
    let hash = argon2::hash_raw(credentials.password.as_bytes(), &salt, &argon_config).unwrap();
    let new_user = NewUser {
        username: &credentials.username,
        password_salt: &salt,
        password_hash: &hash,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)
}

pub fn authenticate_user(credentials: &Credentials, db_conn: &PgConnection) -> Option<User> {
    users::table
        .filter(users::username.eq(&credentials.username))
        .first::<User>(db_conn)
        .optional()
        .unwrap()
        .and_then(|user| {
            let password_matches = argon2::verify_raw(
                credentials.password.as_bytes(),
                &user.password_salt,
                &user.password_hash,
                &argon2_config(),
            )
            .unwrap();

            if password_matches {
                return Some(user);
            } else {
                return None;
            }
        })
}

#[test]
fn test_argon() {
    let credentials = Credentials {
        username: "piepkonijn",
        password: "geheim123",
    };
    let argon_config = argon2_config();

    let salt: [u8; 32] = rand::thread_rng().gen();
    let hash = argon2::hash_raw(credentials.password.as_bytes(), &salt, &argon_config).unwrap();
    let new_user = NewUser {
        username: &credentials.username,
        password_hash: &hash,
        password_salt: &salt,
    };

    let password_matches = argon2::verify_raw(
        credentials.password.as_bytes(),
        &new_user.password_salt,
        &new_user.password_hash,
        &argon2_config(),
    )
    .unwrap();

    assert!(password_matches);
}
