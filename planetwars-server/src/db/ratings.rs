use diesel::{prelude::*, PgConnection, QueryResult};
use serde::{Deserialize, Serialize};

use crate::schema::{bots, ratings};

#[derive(Queryable, Debug, Insertable, PartialEq, Serialize, Deserialize)]
pub struct Rating {
    pub bot_id: i32,
    pub rating: f64,
}

pub fn get_rating(bot_id: i32, db_conn: &PgConnection) -> QueryResult<Option<f64>> {
    ratings::table
        .filter(ratings::bot_id.eq(bot_id))
        .select(ratings::rating)
        .first(db_conn)
        .optional()
}

pub fn set_rating(bot_id: i32, rating: f64, db_conn: &PgConnection) -> QueryResult<usize> {
    diesel::insert_into(ratings::table)
        .values(Rating { bot_id, rating })
        .on_conflict(ratings::bot_id)
        .do_update()
        .set(ratings::rating.eq(rating))
        .execute(db_conn)
}
