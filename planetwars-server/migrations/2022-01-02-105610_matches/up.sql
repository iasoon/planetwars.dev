CREATE TYPE match_state AS ENUM ('playing', 'finished');

CREATE TABLE matches (
    id SERIAL PRIMARY KEY NOT NULL,
    state match_state NOT NULL,
    log_path text NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX match_created_at ON matches(created_at);

CREATE TABLE match_players (
    match_id integer REFERENCES matches(id) NOT NULL,
    player_id integer NOT NULL,
    code_bundle_id integer REFERENCES code_bundles(id) NOT NULL,
    PRIMARY KEY (match_id, player_id)
);