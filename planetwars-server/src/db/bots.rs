use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{bots, code_bundles};
use chrono;

#[derive(Insertable)]
#[table_name = "bots"]
pub struct NewBot<'a> {
    pub owner_id: i32,
    pub name: &'a str,
}

#[derive(Queryable, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
}

pub fn create_bot(new_bot: &NewBot, conn: &PgConnection) -> QueryResult<Bot> {
    diesel::insert_into(bots::table)
        .values(new_bot)
        .get_result(conn)
}

pub fn find_bot(id: i32, conn: &PgConnection) -> QueryResult<Bot> {
    bots::table.find(id).first(conn)
}

pub fn find_bots_by_owner(owner_id: i32, conn: &PgConnection) -> QueryResult<Vec<Bot>> {
    bots::table
        .filter(bots::owner_id.eq(owner_id))
        .get_results(conn)
}

#[derive(Insertable)]
#[table_name = "code_bundles"]
pub struct NewCodeBundle<'a> {
    pub bot_id: i32,
    pub path: &'a str,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct CodeBundle {
    pub id: i32,
    pub bot_id: i32,
    pub path: String,
    pub created_at: chrono::NaiveDateTime,
}

pub fn create_code_bundle(
    new_code_bundle: &NewCodeBundle,
    conn: &PgConnection,
) -> QueryResult<CodeBundle> {
    diesel::insert_into(code_bundles::table)
        .values(new_code_bundle)
        .get_result(conn)
}

pub fn find_bot_code_bundles(bot_id: i32, conn: &PgConnection) -> QueryResult<Vec<CodeBundle>> {
    code_bundles::table
        .filter(code_bundles::bot_id.eq(bot_id))
        .get_results(conn)
}
