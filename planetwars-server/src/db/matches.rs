pub use crate::db_types::MatchState;
use chrono::NaiveDateTime;
use diesel::associations::BelongsTo;
use diesel::pg::Pg;
use diesel::sql_types::*;
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use diesel::{Connection, GroupedBy, PgConnection, QueryResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::schema::{bot_versions, bots, maps, match_players, matches};

use super::bots::{Bot, BotVersion};
use super::maps::Map;
use super::match_queries::ListBotMatches;

#[derive(Insertable)]
#[diesel(table_name = matches)]
pub struct NewMatch<'a> {
    pub state: MatchState,
    pub log_path: &'a str,
    pub is_public: bool,
    pub map_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = match_players)]
pub struct NewMatchPlayer {
    /// id of the match this player is in
    pub match_id: i32,
    /// player id within the match
    pub player_id: i32,
    /// id of the bot behind this player
    pub bot_version_id: Option<i32>,
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = matches)]
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
#[diesel(primary_key(match_id, player_id))]
#[diesel(belongs_to(MatchBase, foreign_key = match_id))]
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
    conn: &mut PgConnection,
) -> QueryResult<MatchData> {
    conn.transaction(|conn| {
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
    conn: &mut PgConnection,
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
pub fn list_matches(amount: i64, conn: &mut PgConnection) -> QueryResult<Vec<FullMatchData>> {
    conn.transaction(|conn| {
        let matches = matches::table
            .filter(matches::state.eq(MatchState::Finished))
            .order_by(matches::created_at.desc())
            .limit(amount)
            .get_results::<MatchBase>(conn)?;

        fetch_full_match_data(matches, conn)
    })
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotMatchOutcome {
    Win,
    Loss,
    Tie,
}

pub fn list_public_matches(
    amount: i64,
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
    conn: &mut PgConnection,
) -> QueryResult<Vec<FullMatchData>> {
    conn.transaction(|conn| {
        let query = finished_public_matches_query(before, after).limit(amount);
        let matches = query.get_results::<MatchBase>(conn)?;
        fetch_full_match_data(matches, conn)
    })
}

pub fn list_bot_matches(
    bot_id: i32,
    opponent_id: Option<i32>,
    outcome: Option<BotMatchOutcome>,
    amount: i64,
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
    conn: &mut PgConnection,
) -> QueryResult<Vec<FullMatchData>> {
    let lbm = ListBotMatches {
        bot_id,
        opponent_id,
        outcome,
        before,
        after,
        amount,
    };

    let matches = lbm.get_results::<MatchBase>(conn)?;
    fetch_full_match_data(matches, conn)
}

fn finished_public_matches_query(
    before: Option<NaiveDateTime>,
    after: Option<NaiveDateTime>,
) -> matches::BoxedQuery<'static, Pg> {
    let query = matches::table
        .filter(matches::state.eq(MatchState::Finished))
        .filter(matches::is_public.eq(true))
        .into_boxed();

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

pub fn find_match(id: i32, conn: &mut PgConnection) -> QueryResult<FullMatchData> {
    conn.transaction(|conn| {
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

pub fn find_match_base(id: i32, conn: &mut PgConnection) -> QueryResult<MatchBase> {
    matches::table.find(id).get_result::<MatchBase>(conn)
}

pub enum MatchResult {
    Finished { winner: Option<i32> },
}

pub fn save_match_result(id: i32, result: MatchResult, conn: &mut PgConnection) -> QueryResult<()> {
    let MatchResult::Finished { winner } = result;

    diesel::update(matches::table.find(id))
        .set((
            matches::winner.eq(winner),
            matches::state.eq(MatchState::Finished),
        ))
        .execute(conn)?;
    Ok(())
}

#[derive(QueryableByName)]
pub struct BotStatsRecord {
    #[diesel(sql_type = Text)]
    pub opponent: String,
    #[diesel(sql_type = Text)]
    pub map: String,
    #[diesel(sql_type = Nullable<Bool>)]
    pub win: Option<bool>,
    #[diesel(sql_type = Int8)]
    pub count: i64,
}

pub fn fetch_bot_stats(
    bot_name: &str,
    db_conn: &mut PgConnection,
) -> QueryResult<Vec<BotStatsRecord>> {
    diesel::sql_query(
        "
SELECT opponent, map, win, COUNT(*) as count
FROM (
    SELECT
        opponent_bot.name as opponent,
        maps.name as map,
        (matches.winner = bot_player.player_id) as win
    FROM matches
    JOIN maps
        ON matches.map_id = maps.id
    JOIN match_players bot_player
        ON bot_player.match_id = matches.id
    JOIN bot_versions bot_version
        ON bot_version.id = bot_player.bot_version_id
    JOIN bots bot
        ON bot.id = bot_version.bot_id
    JOIN match_players opponent_player
        ON opponent_player.match_id = matches.id
        AND opponent_player.player_id = 1 - bot_player.player_id
    JOIN bot_versions opponent_version
        ON opponent_version.id = opponent_player.bot_version_id
    LEFT OUTER JOIN bots opponent_bot
        ON opponent_version.bot_id = opponent_bot.id
    WHERE
        matches.state = 'finished'
        AND matches.is_public
        AND bot.name = $1
    ORDER BY
        matches.created_at DESC 
    LIMIT 10000
) bot_matches
GROUP BY opponent, map, win
HAVING opponent IS NOT NULL",
    )
    .bind::<Text, _>(bot_name)
    .load(db_conn)
}
