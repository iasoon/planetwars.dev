CREATE TABLE bots (
    id serial PRIMARY KEY,
    owner_id integer REFERENCES users(id),
    name text NOT NULL
);

CREATE UNIQUE INDEX bots_index ON bots(owner_id, name);

CREATE TABLE code_bundles (
    id serial PRIMARY KEY,
    bot_id integer REFERENCES bots(id),
    path text NOT NULL,
    created_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
);