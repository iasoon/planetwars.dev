use diesel::{prelude::*, PgConnection, QueryResult};
use serde::{Deserialize, Serialize};

use crate::db::bots::Bot;
use crate::schema::{bots, ratings, users};

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

#[derive(Queryable, Serialize, Deserialize)]
pub struct Author {
    id: i32,
    username: String,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct RankedBot {
    pub bot: Bot,
    pub author: Option<Author>,
    pub rating: f64,
}

pub fn get_bot_ranking(db_conn: &PgConnection) -> QueryResult<Vec<RankedBot>> {
    bots::table
        .left_join(users::table)
        .inner_join(ratings::table)
        .select((
            bots::all_columns,
            (users::id, users::username).nullable(),
            ratings::rating,
        ))
        .order_by(ratings::rating.desc())
        .get_results(db_conn)
}
