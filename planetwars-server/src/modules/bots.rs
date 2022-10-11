use std::path::PathBuf;

use diesel::{PgConnection, QueryResult};

use crate::{db, util::gen_alphanumeric, GlobalConfig};

/// Save a string containing bot code as a code bundle.
/// If a bot was provided, set the saved bundle as its active version.
pub fn save_code_string(
    bot_code: &str,
    bot_id: Option<i32>,
    conn: &PgConnection,
    config: &GlobalConfig,
) -> QueryResult<db::bots::BotVersion> {
    let bundle_name = gen_alphanumeric(16);

    let code_bundle_dir = PathBuf::from(&config.bots_directory).join(&bundle_name);
    std::fs::create_dir(&code_bundle_dir).unwrap();
    std::fs::write(code_bundle_dir.join("bot.py"), bot_code).unwrap();

    let new_code_bundle = db::bots::NewBotVersion {
        bot_id,
        code_bundle_path: Some(&bundle_name),
        container_digest: None,
    };
    let version = db::bots::create_bot_version(&new_code_bundle, conn)?;
    // Leave this coupled for now - this is how the behaviour was before.
    // It would be cleaner to separate version setting and bot selection, though.
    if let Some(bot_id) = bot_id {
        db::bots::set_active_version(bot_id, Some(version.id), conn)?;
    }
    Ok(version)
}
