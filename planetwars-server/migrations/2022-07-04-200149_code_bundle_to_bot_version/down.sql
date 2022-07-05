ALTER TABLE bot_versions DROP COLUMN container_digest;
ALTER TABLE bot_versions RENAME COLUMN code_bundle_path to path;
ALTER TABLE bot_versions ALTER COLUMN path SET NOT NULL;
ALTER TABLE bot_versions RENAME TO code_bundles;
