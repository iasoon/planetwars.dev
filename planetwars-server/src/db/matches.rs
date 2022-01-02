use chrono::NaiveDateTime;
use diesel::{BelongingToDsl, QueryDsl, RunQueryDsl};
use diesel::{Connection, GroupedBy, PgConnection, QueryResult};

use crate::schema::{match_players, matches};

#[derive(Insertable)]
#[table_name = "matches"]
pub struct NewMatch<'a> {
    pub log_path: &'a str,
}

#[derive(Insertable)]
#[table_name = "match_players"]
pub struct NewMatchPlayer {
    /// id of the match this player is in
    pub match_id: i32,
    /// id of the bot behind this player
    pub bot_id: i32,
    /// player id within the match
    pub player_id: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "matches"]
pub struct MatchBase {
    pub id: i32,
    pub log_path: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Identifiable, Associations)]
#[primary_key(match_id, player_id)]
#[belongs_to(MatchBase, foreign_key = "match_id")]
pub struct MatchPlayer {
    pub match_id: i32,
    pub bot_id: i32,
    pub player_id: i32,
}

pub struct MatchPlayerData {
    pub bot_id: i32,
}

pub fn create_match(
    match_data: &NewMatch,
    match_players: &[MatchPlayerData],
    conn: &PgConnection,
) -> QueryResult<i32> {
    conn.transaction(|| {
        let match_base = diesel::insert_into(matches::table)
            .values(match_data)
            .get_result::<MatchBase>(conn)?;

        let match_players = match_players
            .iter()
            .enumerate()
            .map(|(num, player_data)| NewMatchPlayer {
                match_id: match_base.id,
                bot_id: player_data.bot_id,
                player_id: num as i32,
            })
            .collect::<Vec<_>>();

        diesel::insert_into(match_players::table)
            .values(&match_players)
            .execute(conn)?;

        Ok(match_base.id)
    })
}

pub struct MatchData {
    pub base: MatchBase,
    pub match_players: Vec<MatchPlayer>,
}

pub fn list_matches(conn: &PgConnection) -> QueryResult<Vec<MatchData>> {
    conn.transaction(|| {
        let matches = matches::table.get_results::<MatchBase>(conn)?;

        let match_players = MatchPlayer::belonging_to(&matches)
            .load::<MatchPlayer>(conn)?
            .grouped_by(&matches);

        let res = matches
            .into_iter()
            .zip(match_players.into_iter())
            .map(|(base, players)| MatchData {
                base,
                match_players: players.into_iter().collect(),
            })
            .collect();

        Ok(res)
    })
}

pub fn find_match(id: i32, conn: &PgConnection) -> QueryResult<MatchData> {
    conn.transaction(|| {
        let match_base = matches::table.find(id).get_result::<MatchBase>(conn)?;

        let match_players = MatchPlayer::belonging_to(&match_base).load::<MatchPlayer>(conn)?;

        let res = MatchData {
            base: match_base,
            match_players,
        };

        Ok(res)
    })
}

pub fn find_mach_base(id: i32, conn: &PgConnection) -> QueryResult<MatchBase> {
    matches::table.find(id).get_result::<MatchBase>(conn)
}
