pub use crate::db_types::MatchState;
use chrono::NaiveDateTime;
use diesel::associations::BelongsTo;
use diesel::{
    BelongingToDsl, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use diesel::{Connection, GroupedBy, PgConnection, QueryResult};

use crate::schema::{bot_versions, bots, match_players, matches};

use super::bots::{Bot, CodeBundle};

#[derive(Insertable)]
#[table_name = "matches"]
pub struct NewMatch<'a> {
    pub state: MatchState,
    pub log_path: &'a str,
}

#[derive(Insertable)]
#[table_name = "match_players"]
pub struct NewMatchPlayer {
    /// id of the match this player is in
    pub match_id: i32,
    /// player id within the match
    pub player_id: i32,
    /// id of the bot behind this player
    pub code_bundle_id: Option<i32>,
}

#[derive(Queryable, Identifiable)]
#[table_name = "matches"]
pub struct MatchBase {
    pub id: i32,
    pub state: MatchState,
    pub log_path: String,
    pub created_at: NaiveDateTime,
    pub winner: Option<i32>,
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
                code_bundle_id: player_data.code_bundle_id,
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

pub fn list_matches(conn: &PgConnection) -> QueryResult<Vec<FullMatchData>> {
    conn.transaction(|| {
        let matches = matches::table.get_results::<MatchBase>(conn)?;

        let match_players = MatchPlayer::belonging_to(&matches)
            .left_join(
                bot_versions::table
                    .on(match_players::code_bundle_id.eq(bot_versions::id.nullable())),
            )
            .left_join(bots::table.on(bot_versions::bot_id.eq(bots::id.nullable())))
            .load::<FullMatchPlayerData>(conn)?
            .grouped_by(&matches);

        let res = matches
            .into_iter()
            .zip(match_players.into_iter())
            .map(|(base, players)| FullMatchData {
                base,
                match_players: players.into_iter().collect(),
            })
            .collect();

        Ok(res)
    })
}

// TODO: maybe unify this with matchdata?
pub struct FullMatchData {
    pub base: MatchBase,
    pub match_players: Vec<FullMatchPlayerData>,
}

#[derive(Queryable)]
// #[primary_key(base.match_id, base::player_id)]
pub struct FullMatchPlayerData {
    pub base: MatchPlayer,
    pub code_bundle: Option<CodeBundle>,
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

        let match_players = MatchPlayer::belonging_to(&match_base)
            .left_join(
                bot_versions::table
                    .on(match_players::code_bundle_id.eq(bot_versions::id.nullable())),
            )
            .left_join(bots::table.on(bot_versions::bot_id.eq(bots::id.nullable())))
            .load::<FullMatchPlayerData>(conn)?;

        let res = FullMatchData {
            base: match_base,
            match_players,
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
