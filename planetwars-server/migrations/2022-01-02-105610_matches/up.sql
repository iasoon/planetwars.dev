CREATE TYPE match_state AS ENUM ('playing', 'ended');

CREATE TABLE matches (
    id SERIAL PRIMARY KEY NOT NULL,
    state match_state NOT NULL,
    log_path text NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX match_created_at ON matches(created_at);

CREATE TABLE match_players (
    match_id integer REFERENCES matches(id) NOT NULL,
    bot_id integer REFERENCES bots(id) NOT NULL,
    player_id integer NOT NULL,
    PRIMARY KEY (match_id, player_id)
);