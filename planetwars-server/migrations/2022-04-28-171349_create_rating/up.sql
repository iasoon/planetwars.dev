-- Your SQL goes here
-- this table could later be expanded to include more information,
-- such as rating state (eg. number of matches played) or scope (eg. map)
create table ratings (
    bot_id integer PRIMARY KEY REFERENCES bots(id),
    rating float NOT NULL
)