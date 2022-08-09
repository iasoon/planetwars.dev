use crate::schema::users;
use argon2;
use diesel::{prelude::*, PgConnection};
use rand::Rng;
use serde::Deserialize;

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
    pub id: i32,
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

pub fn hash_password(password: &str) -> (Vec<u8>, [u8; 32]) {
    let argon_config = argon2_config();
    let salt: [u8; 32] = rand::thread_rng().gen();
    let hash = argon2::hash_raw(password.as_bytes(), &salt, &argon_config).unwrap();

    (hash, salt)
}

pub fn create_user(credentials: &Credentials, conn: &PgConnection) -> QueryResult<User> {
    let (hash, salt) = hash_password(&credentials.password);

    let new_user = NewUser {
        username: credentials.username,
        password_salt: &salt,
        password_hash: &hash,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)
}

pub fn find_user(user_id: i32, db_conn: &PgConnection) -> QueryResult<User> {
    users::table
        .filter(users::id.eq(user_id))
        .first::<User>(db_conn)
}

pub fn find_user_by_name(username: &str, db_conn: &PgConnection) -> QueryResult<User> {
    users::table
        .filter(users::username.eq(username))
        .first::<User>(db_conn)
}

pub fn set_user_password(credentials: Credentials, db_conn: &PgConnection) -> QueryResult<()> {
    let (hash, salt) = hash_password(&credentials.password);

    let n_changes = diesel::update(users::table.filter(users::username.eq(&credentials.username)))
        .set((
            users::password_salt.eq(salt.as_slice()),
            users::password_hash.eq(hash.as_slice()),
        ))
        .execute(db_conn)?;
    if n_changes == 0 {
        Err(diesel::result::Error::NotFound)
    } else {
        Ok(())
    }
}

pub fn authenticate_user(credentials: &Credentials, db_conn: &PgConnection) -> Option<User> {
    find_user_by_name(credentials.username, db_conn)
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
                Some(user)
            } else {
                None
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
        username: credentials.username,
        password_hash: &hash,
        password_salt: &salt,
    };

    let password_matches = argon2::verify_raw(
        credentials.password.as_bytes(),
        new_user.password_salt,
        new_user.password_hash,
        &argon2_config(),
    )
    .unwrap();

    assert!(password_matches);
}
