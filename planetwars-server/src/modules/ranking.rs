use crate::{db::bots::Bot, DbPool};

use crate::db;
use diesel::{PgConnection, QueryResult};
use crate::modules::matches::{MatchPlayer, RunMatch};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::mem;
use std::time::{Duration, Instant};
use tokio;

const RANKER_INTERVAL: u64 = 60;

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
        recalculate_ratings(&db_conn).expect("could not recalculate ratings");
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
    run_match
        .spawn(db_pool.clone())
        .await
        .expect("running match failed");
}

fn recalculate_ratings(db_conn: &PgConnection) -> QueryResult<()> {
    let start = Instant::now();
    let match_stats = fetch_match_stats(db_conn)?;
    let ratings = estimate_ratings_from_stats(match_stats);

    for (bot_id, rating) in ratings {
        db::ratings::set_rating(bot_id, rating, db_conn).expect("could not update bot rating");
    }
    let elapsed = Instant::now() - start;
    // TODO: set up proper logging infrastructure
    println!("computed ratings in {} ms", elapsed.subsec_millis());
    Ok(())
}

#[derive(Default)]
struct MatchStats {
    total_score: f64,
    num_matches: usize,
}

fn fetch_match_stats(db_conn: &PgConnection) -> QueryResult<HashMap<(i32, i32), MatchStats>> {
    let matches = db::matches::list_matches(db_conn)?;

    let mut match_stats = HashMap::<(i32, i32), MatchStats>::new();
    for m in matches {
        if m.match_players.len() != 2 {
            continue;
        }
        let (mut a_id, mut b_id) = match (&m.match_players[0].bot, &m.match_players[1].bot) {
            (Some(ref a), Some(ref b)) => (a.id, b.id),
            _ => continue,
        };
        // score of player a
        let mut score = match m.base.winner {
            None => 0.5,
            Some(0) => 1.0,
            Some(1) => 0.0,
            _ => panic!("invalid winner"),
        };

        // put players in canonical order: smallest id first
        if b_id < a_id {
            mem::swap(&mut a_id, &mut b_id);
            score = 1.0 - score;
        }

        let entry = match_stats.entry((a_id, b_id)).or_default();
        entry.num_matches += 1;
        entry.total_score += score;
    }
    Ok(match_stats)
}

/// Tokenizes player ids to a set of consecutive numbers
struct PlayerTokenizer {
    id_to_ix: HashMap<i32, usize>,
    ids: Vec<i32>,
}

impl PlayerTokenizer {
    fn new() -> Self {
        PlayerTokenizer {
            id_to_ix: HashMap::new(),
            ids: Vec::new(),
        }
    }

    fn tokenize(&mut self, id: i32) -> usize {
        match self.id_to_ix.get(&id) {
            Some(&ix) => ix,
            None => {
                let ix = self.ids.len();
                self.ids.push(id);
                self.id_to_ix.insert(id, ix);
                ix
            }
        }
    }

    fn detokenize(&self, ix: usize) -> i32 {
        self.ids[ix]
    }

    fn player_count(&self) -> usize {
        self.ids.len()
    }
}

fn sigmoid(logit: f64) -> f64 {
    1.0 / (1.0 + (-logit).exp())
}

fn estimate_ratings_from_stats(match_stats: HashMap<(i32, i32), MatchStats>) -> Vec<(i32, f64)> {
    // map player ids to player indexes in the ratings array
    let mut input_records = Vec::<RatingInputRecord>::with_capacity(match_stats.len());
    let mut player_tokenizer = PlayerTokenizer::new();

    for ((a_id, b_id), stats) in match_stats.into_iter() {
        input_records.push(RatingInputRecord {
            p1_ix: player_tokenizer.tokenize(a_id),
            p2_ix: player_tokenizer.tokenize(b_id),
            score: stats.total_score / stats.num_matches as f64,
            weight: stats.num_matches as f64,
        })
    }

    let mut ratings = vec![0f64; player_tokenizer.player_count()];
    // TODO: fetch these from config
    let params = OptimizeRatingsParams::default();
    optimize_ratings(&mut ratings, &input_records, &params);

    ratings
        .into_iter()
        .enumerate()
        .map(|(ix, rating)| {
            (
                player_tokenizer.detokenize(ix),
                rating * 100f64 / 10f64.ln(),
            )
        })
        .collect()
}

struct RatingInputRecord {
    /// index of first player
    p1_ix: usize,
    /// index of secord player
    p2_ix: usize,
    /// score of player 1 (= 1 - score of player 2)
    score: f64,
    /// weight of this record
    weight: f64,
}

struct OptimizeRatingsParams {
    tolerance: f64,
    learning_rate: f64,
    max_iterations: usize,
    regularization_weight: f64,
}

impl Default for OptimizeRatingsParams {
    fn default() -> Self {
        OptimizeRatingsParams {
            tolerance: 10f64.powi(-8),
            learning_rate: 0.1,
            max_iterations: 10_000,
            regularization_weight: 10.0,
        }
    }
}

fn optimize_ratings(
    ratings: &mut [f64],
    input_records: &[RatingInputRecord],
    params: &OptimizeRatingsParams,
) {
    let total_weight =
        params.regularization_weight + input_records.iter().map(|r| r.weight).sum::<f64>();

    for _iteration in 0..params.max_iterations {
        let mut gradients = vec![0f64; ratings.len()];

        // calculate gradients
        for record in input_records.iter() {
            let predicted = sigmoid(ratings[record.p1_ix] - ratings[record.p2_ix]);
            let gradient = record.weight * (predicted - record.score);
            gradients[record.p1_ix] += gradient;
            gradients[record.p2_ix] -= gradient;
        }

        // apply update step
        let mut converged = true;
        for (rating, gradient) in ratings.iter_mut().zip(&gradients) {
            let update = params.learning_rate * (gradient + params.regularization_weight * *rating)
                / total_weight;
            if update > params.tolerance {
                converged = false;
            }
            *rating -= update;
        }

        if converged {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_close(a: f64, b: f64) -> bool {
        (a - b).abs() < 10f64.powi(-6)
    }

    #[test]
    fn test_optimize_ratings() {
        let input_records = vec![RatingInputRecord {
            p1_ix: 0,
            p2_ix: 1,
            score: 0.8,
            weight: 1.0,
        }];

        let mut ratings = vec![0.0; 2];
        optimize_ratings(
            &mut ratings,
            &input_records,
            &OptimizeRatingsParams {
                regularization_weight: 0.0,
                ..Default::default()
            },
        );
        assert!(is_close(sigmoid(ratings[0] - ratings[1]), 0.8));
    }

    #[test]
    fn test_optimize_ratings_weight() {
        let input_records = vec![
            RatingInputRecord {
                p1_ix: 0,
                p2_ix: 1,
                score: 1.0,
                weight: 1.0,
            },
            RatingInputRecord {
                p1_ix: 1,
                p2_ix: 0,
                score: 1.0,
                weight: 3.0,
            },
        ];

        let mut ratings = vec![0.0; 2];
        optimize_ratings(
            &mut ratings,
            &input_records,
            &OptimizeRatingsParams {
                regularization_weight: 0.0,
                ..Default::default()
            },
        );
        assert!(is_close(sigmoid(ratings[0] - ratings[1]), 0.25));
    }

    #[test]
    fn test_optimize_ratings_regularization() {
        let input_records = vec![RatingInputRecord {
            p1_ix: 0,
            p2_ix: 1,
            score: 0.8,
            weight: 100.0,
        }];

        let mut ratings = vec![0.0; 2];
        optimize_ratings(
            &mut ratings,
            &input_records,
            &OptimizeRatingsParams {
                regularization_weight: 1.0,
                ..Default::default()
            },
        );
        let predicted = sigmoid(ratings[0] - ratings[1]);
        assert!(0.5 < predicted && predicted < 0.8);
    }
}
