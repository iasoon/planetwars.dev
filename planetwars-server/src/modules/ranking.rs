use crate::{db::bots::Bot, DbPool};

use crate::db;
use diesel::PgConnection;
use rand::seq::SliceRandom;
use std::time::Duration;
use tokio;

pub async fn run_ranker(db_pool: DbPool) {
    // TODO: make this configurable
    // play at most one match every n seconds
    let mut interval = tokio::time::interval(Duration::from_secs(10));
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
        play_match(selected_bots, db_pool.clone()).await;
    }
}

async fn play_match(selected_bots: Vec<Bot>, db_pool: DbPool) {}
