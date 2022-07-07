use std::path::PathBuf;

use diesel::{PgConnection, QueryResult};

use crate::{db, util::gen_alphanumeric, BOTS_DIR};

pub fn save_code_bundle(
    bot_code: &str,
    bot_id: Option<i32>,
    conn: &PgConnection,
) -> QueryResult<db::bots::BotVersion> {
    let bundle_name = gen_alphanumeric(16);

    let code_bundle_dir = PathBuf::from(BOTS_DIR).join(&bundle_name);
    std::fs::create_dir(&code_bundle_dir).unwrap();
    std::fs::write(code_bundle_dir.join("bot.py"), bot_code).unwrap();

    let new_code_bundle = db::bots::NewBotVersion {
        bot_id,
        code_bundle_path: Some(&bundle_name),
        container_digest: None,
    };
    db::bots::create_bot_version(&new_code_bundle, conn)
}
