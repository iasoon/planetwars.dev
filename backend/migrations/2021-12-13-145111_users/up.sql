CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    username VARCHAR(52) NOT NULL,
    password_salt BYTEA NOT NULL,
    password_hash BYTEA NOT NULL
);