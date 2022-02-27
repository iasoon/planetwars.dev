CREATE TABLE bots (
    id serial PRIMARY KEY,
    owner_id integer REFERENCES users(id),
    name text UNIQUE NOT NULL
);

CREATE TABLE code_bundles (
    id serial PRIMARY KEY,
    bot_id integer REFERENCES bots(id),
    path text NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX code_bundles_bot_id_index ON code_bundles(bot_id);
CREATE INDEX code_bundles_created_at_index ON code_bundles(created_at);