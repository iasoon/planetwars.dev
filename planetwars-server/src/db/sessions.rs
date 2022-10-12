use super::users::User;
use crate::schema::{sessions, users};
use base64;
use diesel::PgConnection;
use diesel::{insert_into, prelude::*, Insertable, RunQueryDsl};
use rand::{self, Rng};

#[derive(Insertable)]
#[diesel(table_name = sessions)]
struct NewSession {
    token: String,
    user_id: i32,
}

#[derive(Queryable, Debug, PartialEq)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
}

pub fn create_session(user: &User, conn: &mut PgConnection) -> Session {
    let new_session = NewSession {
        token: gen_session_token(),
        user_id: user.id,
    };

    insert_into(sessions::table)
        .values(&new_session)
        .get_result::<Session>(conn)
        .unwrap()
}

pub fn find_user_by_session(token: &str, conn: &mut PgConnection) -> QueryResult<(Session, User)> {
    sessions::table
        .inner_join(users::table)
        .filter(sessions::token.eq(&token))
        .first::<(Session, User)>(conn)
}

pub fn gen_session_token() -> String {
    let mut rng = rand::thread_rng();
    let token: [u8; 32] = rng.gen();
    base64::encode(&token)
}
