CREATE TABLE sessions (
    id serial PRIMARY KEY,
    user_id integer REFERENCES users(id) NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE
)