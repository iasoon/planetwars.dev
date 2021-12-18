use std::future::Future;

use diesel::RunQueryDsl;
use mozaic4_backend::DbConn;
use rocket::{http::Header, local::asynchronous::Client};

// We use a lock to synchronize between tests so DB operations don't collide.
// For now. In the future, we'll have a nice way to run each test in a DB
// transaction so we can regain concurrency.
static DB_LOCK: parking_lot::Mutex<()> = parking_lot::const_mutex(());

async fn reset_db(db: &DbConn) {
    db.run(|conn| {
        diesel::sql_query(
            r#"
            TRUNCATE TABLE users, sessions,
            bots, code_bundles"#,
        )
        .execute(conn)
        .expect("drop all tables");
    })
    .await
}

pub async fn run_test<F, R>(test_closure: F)
where
    F: FnOnce(Client, DbConn) -> R,
    R: Future<Output = ()>,
{
    let _lock = DB_LOCK.lock();

    let client = Client::untracked(mozaic4_backend::rocket())
        .await
        .expect("failed to create test client");
    let db = mozaic4_backend::DbConn::get_one(client.rocket())
        .await
        .expect("failed to get db connection");

    // make sure we start with a clean DB
    reset_db(&db).await;

    test_closure(client, db).await;
}

pub struct BearerAuth {
    token: String,
}

impl BearerAuth {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl<'a> Into<Header<'a>> for BearerAuth {
    fn into(self) -> Header<'a> {
        Header::new("Authorization", format!("Bearer {}", self.token))
    }
}
