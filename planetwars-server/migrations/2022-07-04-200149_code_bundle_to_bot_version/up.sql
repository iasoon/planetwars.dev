ALTER TABLE code_bundles RENAME TO bot_versions;
ALTER TABLE bot_versions RENAME COLUMN path to code_bundle_path;
ALTER TABLE bot_versions ALTER COLUMN code_bundle_path DROP NOT NULL;
ALTER TABLE bot_versions ADD COLUMN container_digest TEXT;

ALTER TABLE match_players RENAME COLUMN code_bundle_id TO bot_version_id;
