use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{bot_versions, bots};
use chrono;

#[derive(Insertable)]
#[diesel(table_name = bots)]
pub struct NewBot<'a> {
    pub owner_id: Option<i32>,
    pub name: &'a str,
}

#[derive(Queryable, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    pub id: i32,
    pub owner_id: Option<i32>,
    pub name: String,
    pub active_version: Option<i32>,
}

pub fn create_bot(new_bot: &NewBot, conn: &mut PgConnection) -> QueryResult<Bot> {
    diesel::insert_into(bots::table)
        .values(new_bot)
        .get_result(conn)
}

pub fn find_bot(id: i32, conn: &mut PgConnection) -> QueryResult<Bot> {
    bots::table.find(id).first(conn)
}

pub fn find_bots_by_owner(owner_id: i32, conn: &mut PgConnection) -> QueryResult<Vec<Bot>> {
    bots::table
        .filter(bots::owner_id.eq(owner_id))
        .get_results(conn)
}

pub fn find_bot_by_name(name: &str, conn: &mut PgConnection) -> QueryResult<Bot> {
    bots::table.filter(bots::name.eq(name)).first(conn)
}

pub fn find_bot_with_version_by_name(
    bot_name: &str,
    conn: &mut PgConnection,
) -> QueryResult<(Bot, BotVersion)> {
    bots::table
        .inner_join(bot_versions::table.on(bots::active_version.eq(bot_versions::id.nullable())))
        .filter(bots::name.eq(bot_name))
        .first(conn)
}

pub fn all_active_bots_with_version(
    conn: &mut PgConnection,
) -> QueryResult<Vec<(Bot, BotVersion)>> {
    bots::table
        .inner_join(bot_versions::table.on(bots::active_version.eq(bot_versions::id.nullable())))
        .get_results(conn)
}

pub fn find_all_bots(conn: &mut PgConnection) -> QueryResult<Vec<Bot>> {
    bots::table.get_results(conn)
}

/// Find all bots that have an associated active version.
/// These are the bots that can be run.
pub fn find_active_bots(conn: &mut PgConnection) -> QueryResult<Vec<Bot>> {
    bots::table
        .filter(bots::active_version.is_not_null())
        .get_results(conn)
}

#[derive(Insertable)]
#[diesel(table_name = bot_versions)]
pub struct NewBotVersion<'a> {
    pub bot_id: Option<i32>,
    pub code_bundle_path: Option<&'a str>,
    pub container_digest: Option<&'a str>,
}

#[derive(Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct BotVersion {
    pub id: i32,
    pub bot_id: Option<i32>,
    pub code_bundle_path: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub container_digest: Option<String>,
}

pub fn create_bot_version(
    new_bot_version: &NewBotVersion,
    conn: &mut PgConnection,
) -> QueryResult<BotVersion> {
    diesel::insert_into(bot_versions::table)
        .values(new_bot_version)
        .get_result(conn)
}

pub fn set_active_version(
    bot_id: i32,
    version_id: Option<i32>,
    conn: &mut PgConnection,
) -> QueryResult<()> {
    diesel::update(bots::table.filter(bots::id.eq(bot_id)))
        .set(bots::active_version.eq(version_id))
        .execute(conn)?;
    Ok(())
}

pub fn find_bot_version(version_id: i32, conn: &mut PgConnection) -> QueryResult<BotVersion> {
    bot_versions::table
        .filter(bot_versions::id.eq(version_id))
        .first(conn)
}

pub fn find_bot_versions(bot_id: i32, conn: &mut PgConnection) -> QueryResult<Vec<BotVersion>> {
    bot_versions::table
        .filter(bot_versions::bot_id.eq(bot_id))
        .get_results(conn)
}
