-- Your SQL goes here
ALTER TABLE bots ADD COLUMN active_version INTEGER REFERENCES bot_versions(id);

-- set most recent bot verison as active
UPDATE bots
SET active_version = most_recent.id
FROM (
    SELECT DISTINCT ON (bot_id) id, bot_id
    FROM bot_versions
    ORDER BY bot_id, created_at DESC
    ) most_recent
WHERE bots.id = most_recent.bot_id;