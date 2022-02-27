use std::path::PathBuf;

use diesel::{PgConnection, QueryResult};

use crate::{db, util::gen_alphanumeric, BOTS_DIR};

pub fn save_code_bundle(
    bot_code: &str,
    bot_id: Option<i32>,
    conn: &PgConnection,
) -> QueryResult<db::bots::CodeBundle> {
    let bundle_name = gen_alphanumeric(16);

    let code_bundle_dir = PathBuf::from(BOTS_DIR).join(&bundle_name);
    std::fs::create_dir(&code_bundle_dir).unwrap();
    std::fs::write(code_bundle_dir.join("bot.py"), bot_code).unwrap();

    let new_code_bundle = db::bots::NewCodeBundle {
        bot_id,
        path: &bundle_name,
    };
    db::bots::create_code_bundle(&new_code_bundle, conn)
}
