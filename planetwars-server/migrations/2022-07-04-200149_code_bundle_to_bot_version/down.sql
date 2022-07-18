ALTER TABLE match_players RENAME COLUMN bot_version_id TO code_bundle_id;

ALTER TABLE bot_versions DROP COLUMN container_digest;
ALTER TABLE bot_versions RENAME COLUMN code_bundle_path TO path;
ALTER TABLE bot_versions ALTER COLUMN path SET NOT NULL;
ALTER TABLE bot_versions RENAME TO code_bundles;
