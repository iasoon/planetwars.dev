use super::matches::BotMatchOutcome;
use crate::schema::matches;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::query_builder::{AstPass, Query, QueryFragment, QueryId};
use diesel::sql_types::*;
use diesel::{PgConnection, QueryResult, RunQueryDsl};

pub struct ListBotMatches {
    pub bot_id: i32,
    pub had_errors: Option<bool>,
    pub outcome: Option<BotMatchOutcome>,

    pub opponent_id: Option<i32>,

    // pagination options
    pub before: Option<NaiveDateTime>,
    pub after: Option<NaiveDateTime>,
    pub amount: i64,
}

impl QueryFragment<Pg> for ListBotMatches {
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();

        out.push_sql("SELECT matches.* FROM matches");
        out.push_sql(" JOIN (");
        out.push_sql(concat!(
            "SELECT match_id, player_id, bot_version_id, bot_id ",
            "FROM match_players ",
            "JOIN bot_versions ON match_players.bot_version_id = bot_versions.id ",
            "WHERE bot_id = "
        ));
        out.push_bind_param::<Integer, _>(&self.bot_id)?;

        if let Some(had_errors) = self.had_errors.as_ref() {
            out.push_sql(" AND match_players.had_errors = ");
            out.push_bind_param::<Bool, _>(had_errors)?;
        }

        out.push_sql(") main_player ON matches.id = main_player.match_id");

        if let Some(opponent_id) = self.opponent_id.as_ref() {
            out.push_sql(" JOIN (");
            out.push_sql(concat!(
                "SELECT match_id, player_id, bot_version_id, bot_id ",
                "FROM match_players ",
                "JOIN bot_versions ON match_players.bot_version_id = bot_versions.id ",
                "WHERE bot_id = "
            ));
            out.push_bind_param::<Integer, _>(opponent_id)?;

            out.push_sql(") other_player ON matches.id = other_player.match_id");
        }

        out.push_sql(" WHERE matches.state = 'finished' AND matches.is_public = true");
        if let Some(outcome) = self.outcome.as_ref() {
            match outcome {
                BotMatchOutcome::Win => {
                    out.push_sql(" AND matches.winner = main_player.player_id");
                }
                BotMatchOutcome::Loss => {
                    out.push_sql(" AND matches.winner <> main_player.player_id");
                }
                BotMatchOutcome::Tie => {
                    out.push_sql(" AND matches.winner IS NULL");
                }
            }
        }
        if let Some(before) = self.before.as_ref() {
            out.push_sql(" AND matches.created_at < ");
            out.push_bind_param::<Timestamp, _>(before)?;
            out.push_sql(" ORDER BY matches.created_at DESC");
        } else if let Some(after) = self.after.as_ref() {
            out.push_sql(" AND matches.created_at > ");
            out.push_bind_param::<Timestamp, _>(after)?;
            out.push_sql(" ORDER BY matches.created_at ASC");
        }
        out.push_sql(" LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.amount)?;

        Ok(())
    }
}

impl Query for ListBotMatches {
    type SqlType = matches::SqlType;
}

impl QueryId for ListBotMatches {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl RunQueryDsl<PgConnection> for ListBotMatches {}
