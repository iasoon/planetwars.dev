use crate::{db::bots::Bot, DbPool};

use crate::db;
use crate::modules::matches::{MatchPlayer, RunMatch};
use rand::seq::SliceRandom;
use std::time::Duration;
use tokio;

const RANKER_INTERVAL: u64 = 60;
const START_RATING: f64 = 0.0;
const SCALE: f64 = 100.0;
const MAX_UPDATE: f64 = 0.1;

pub async fn run_ranker(db_pool: DbPool) {
    // TODO: make this configurable
    // play at most one match every n seconds
    let mut interval = tokio::time::interval(Duration::from_secs(RANKER_INTERVAL));
    let db_conn = db_pool
        .get()
        .await
        .expect("could not get database connection");
    loop {
        interval.tick().await;
        let bots = db::bots::find_all_bots(&db_conn).unwrap();
        if bots.len() < 2 {
            // not enough bots to play a match
            continue;
        }
        let selected_bots: Vec<Bot> = {
            let mut rng = &mut rand::thread_rng();
            bots.choose_multiple(&mut rng, 2).cloned().collect()
        };
        play_ranking_match(selected_bots, db_pool.clone()).await;
    }
}

async fn play_ranking_match(selected_bots: Vec<Bot>, db_pool: DbPool) {
    let db_conn = db_pool.get().await.expect("could not get db pool");
    let mut code_bundles = Vec::new();
    for bot in &selected_bots {
        let code_bundle = db::bots::active_code_bundle(bot.id, &db_conn)
            .expect("could not get active code bundle");
        code_bundles.push(code_bundle);
    }

    let players = code_bundles
        .iter()
        .map(MatchPlayer::from_code_bundle)
        .collect::<Vec<_>>();

    let mut run_match = RunMatch::from_players(players);
    run_match
        .store_in_database(&db_conn)
        .expect("could not store match in db");
    let outcome = run_match
        .spawn(db_pool.clone())
        .await
        .expect("running match failed");

    let mut ratings = Vec::new();
    for bot in &selected_bots {
        let rating = db::ratings::get_rating(bot.id, &db_conn)
            .expect("could not get bot rating")
            .unwrap_or(START_RATING);
        ratings.push(rating);
    }

    // simple elo rating

    let scores = match outcome.winner {
        None => vec![0.5; 2],
        Some(player_num) => {
            // TODO: please get rid of this offset
            let player_ix = player_num - 1;
            let mut scores = vec![0.0; 2];
            scores[player_ix] = 1.0;
            scores
        }
    };

    for i in 0..2 {
        let j = 1 - i;

        let scaled_difference = (ratings[j] - ratings[i]) / SCALE;
        let expected = 1.0 / (1.0 + 10f64.powf(scaled_difference));
        let new_rating = ratings[i] + MAX_UPDATE * (scores[i] - expected);
        db::ratings::set_rating(selected_bots[i].id, new_rating, &db_conn)
            .expect("could not update bot rating");
    }
}
