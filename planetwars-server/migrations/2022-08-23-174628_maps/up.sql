CREATE TABLE maps (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    file_path TEXT NOT NULL
);

ALTER TABLE matches ADD COLUMN map_id INTEGER REFERENCES maps(id);