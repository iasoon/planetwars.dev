pub use crate::db_types::MatchState;
use chrono::NaiveDateTime;
use diesel::associations::BelongsTo;
use diesel::pg::Pg;
use diesel::query_builder::BoxedSelectStatement;
use diesel::query_source::{AppearsInFromClause, Once};
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use diesel::{Connection, GroupedBy, PgConnection, QueryResult};
use std::collections::{HashMap, HashSet};

use crate::schema::{bot_versions, bots, maps, match_players, matches};

use super::bots::{Bot, BotVersion};
use super::maps::Map;

#[derive(Insertable)]
#[table_name = "matches"]
pub struct NewMatch<'a> {
    pub state: MatchState,
    pub log_path: &'a str,
    pub is_public: bool,
    pub map_id: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "match_players"]
pub struct NewMatchPlayer {
    /// id of the match this player is in
    pub match_id: i32,
    /// player id within the match
    pub player_id: i32,
    /// id of the bot behind this player
    pub bot_version_id: Option<i32>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "matches"]
pub struct MatchBase {
    pub id: i32,
    pub state: MatchState,
    pub log_path: String,
    pub created_at: NaiveDateTime,
    pub winner: Option<i32>,
    pub is_public: bool,
    pub map_id: Option<i32>,
}

#[derive(Queryable, Identifiable, Associations, Clone)]
#[primary_key(match_id, player_id)]
#[belongs_to(MatchBase, foreign_key = "match_id")]
pub struct MatchPlayer {
    pub match_id: i32,
    pub player_id: i32,
    pub code_bundle_id: Option<i32>,
}

pub struct MatchPlayerData {
    pub code_bundle_id: Option<i32>,
}

pub fn create_match(
    new_match_base: &NewMatch,
    new_match_players: &[MatchPlayerData],
    conn: &PgConnection,
) -> QueryResult<MatchData> {
    conn.transaction(|| {
        let match_base = diesel::insert_into(matches::table)
            .values(new_match_base)
            .get_result::<MatchBase>(conn)?;

        let new_match_players = new_match_players
            .iter()
            .enumerate()
            .map(|(num, player_data)| NewMatchPlayer {
                match_id: match_base.id,
                player_id: num as i32,
                bot_version_id: player_data.code_bundle_id,
            })
            .collect::<Vec<_>>();

        let match_players = diesel::insert_into(match_players::table)
            .values(&new_match_players)
            .get_results::<MatchPlayer>(conn)?;

        Ok(MatchData {
            base: match_base,
            match_players,
        })
    })
}

pub struct MatchData {
    pub base: MatchBase,
    pub match_players: Vec<MatchPlayer>,
}

/// Add player information to MatchBase instances
fn fetch_full_match_data(
    matches: Vec<MatchBase>,
    conn: &PgConnection,
) -> QueryResult<Vec<FullMatchData>> {
    let map_ids: HashSet<i32> = matches.iter().filter_map(|m| m.map_id).collect();

    let maps_by_id: HashMap<i32, Map> = maps::table
        .filter(maps::id.eq_any(map_ids))
        .load::<Map>(conn)?
        .into_iter()
        .map(|m| (m.id, m))
        .collect();

    let match_players = MatchPlayer::belonging_to(&matches)
        .left_join(
            bot_versions::table.on(match_players::bot_version_id.eq(bot_versions::id.nullable())),
        )
        .left_join(bots::table.on(bot_versions::bot_id.eq(bots::id.nullable())))
        .order_by((
            match_players::match_id.asc(),
            match_players::player_id.asc(),
        ))
        .load::<FullMatchPlayerData>(conn)?
        .grouped_by(&matches);

    let res = matches
        .into_iter()
        .zip(match_players.into_iter())
        .map(|(base, players)| FullMatchData {
            match_players: players.into_iter().collect(),
            map: base
                .map_id
                .and_then(|map_id| maps_by_id.get(&map_id).cloned()),
            base,
        })
        .collect();

    Ok(res)
}

// TODO: this method should disappear
pub fn list_matches(amount: i64, conn: &PgConnection) -> QueryResult<Vec<FullMatchData>> {
    conn.transaction(|| {
        let matches = matches::table
            .order_by(matches::created_at.desc())
            .limit(amount)
            .get_results::<MatchBase>(conn)?;

        fetch_full_match_data(matches, conn)
    })
}

pub fn list_public_matches(
    amount: i64,
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
    conn: &PgConnection,
) -> QueryResult<Vec<FullMatchData>> {
    conn.transaction(|| {
        // TODO: how can this common logic be abstracted?
        let query = matches::table
            .filter(matches::is_public.eq(true))
            .into_boxed();

        let matches =
            select_matches_page(query, amount, before, after).get_results::<MatchBase>(conn)?;
        fetch_full_match_data(matches, conn)
    })
}

pub fn list_bot_matches(
    bot_id: i32,
    amount: i64,
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
    conn: &PgConnection,
) -> QueryResult<Vec<FullMatchData>> {
    let query = matches::table
        .filter(matches::is_public.eq(true))
        .order_by(matches::created_at.desc())
        .inner_join(match_players::table)
        .inner_join(
            bot_versions::table.on(match_players::bot_version_id.eq(bot_versions::id.nullable())),
        )
        .filter(bot_versions::bot_id.eq(bot_id))
        .select(matches::all_columns)
        .into_boxed();

    let matches =
        select_matches_page(query, amount, before, after).get_results::<MatchBase>(conn)?;
    fetch_full_match_data(matches, conn)
}

fn select_matches_page<QS>(
    query: BoxedSelectStatement<'static, matches::SqlType, QS, Pg>,
    amount: i64,
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
) -> BoxedSelectStatement<'static, matches::SqlType, QS, Pg>
where
    QS: AppearsInFromClause<matches::table, Count = Once>,
{
    // TODO: this is not nice. Replace this with proper cursor logic.
    match (before, after) {
        (None, None) => query.order_by(matches::created_at.desc()),
        (Some(before), None) => query
            .filter(matches::created_at.lt(before))
            .order_by(matches::created_at.desc()),
        (None, Some(after)) => query
            .filter(matches::created_at.gt(after))
            .order_by(matches::created_at.asc()),
        (Some(before), Some(after)) => query
            .filter(matches::created_at.lt(before))
            .filter(matches::created_at.gt(after))
            .order_by(matches::created_at.desc()),
    }
    .limit(amount)
}

// TODO: maybe unify this with matchdata?
pub struct FullMatchData {
    pub base: MatchBase,
    pub map: Option<Map>,
    pub match_players: Vec<FullMatchPlayerData>,
}

#[derive(Queryable)]
// #[primary_key(base.match_id, base::player_id)]
pub struct FullMatchPlayerData {
    pub base: MatchPlayer,
    pub bot_version: Option<BotVersion>,
    pub bot: Option<Bot>,
}

impl BelongsTo<MatchBase> for FullMatchPlayerData {
    type ForeignKey = i32;
    type ForeignKeyColumn = match_players::match_id;

    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        Some(&self.base.match_id)
    }

    fn foreign_key_column() -> Self::ForeignKeyColumn {
        match_players::match_id
    }
}

pub fn find_match(id: i32, conn: &PgConnection) -> QueryResult<FullMatchData> {
    conn.transaction(|| {
        let match_base = matches::table.find(id).get_result::<MatchBase>(conn)?;

        let map = match match_base.map_id {
            None => None,
            Some(map_id) => Some(super::maps::find_map(map_id, conn)?),
        };

        let match_players = MatchPlayer::belonging_to(&match_base)
            .left_join(
                bot_versions::table
                    .on(match_players::bot_version_id.eq(bot_versions::id.nullable())),
            )
            .left_join(bots::table.on(bot_versions::bot_id.eq(bots::id.nullable())))
            .order_by(match_players::player_id.asc())
            .load::<FullMatchPlayerData>(conn)?;

        let res = FullMatchData {
            base: match_base,
            match_players,
            map,
        };

        Ok(res)
    })
}

pub fn find_match_base(id: i32, conn: &PgConnection) -> QueryResult<MatchBase> {
    matches::table.find(id).get_result::<MatchBase>(conn)
}

pub enum MatchResult {
    Finished { winner: Option<i32> },
}

pub fn save_match_result(id: i32, result: MatchResult, conn: &PgConnection) -> QueryResult<()> {
    let MatchResult::Finished { winner } = result;

    diesel::update(matches::table.find(id))
        .set((
            matches::winner.eq(winner),
            matches::state.eq(MatchState::Finished),
        ))
        .execute(conn)?;
    Ok(())
}
