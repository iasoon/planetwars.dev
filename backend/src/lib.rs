#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod db;
pub mod routes;
pub mod schema;

#[database("postgresql_database")]
pub struct DbConn(diesel::PgConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                routes::users::register,
                routes::users::login,
                routes::users::current_user,
                routes::bots::create_bot,
                routes::bots::get_bot,
                routes::bots::upload_bot_code,
            ],
        )
        .attach(DbConn::fairing())
}
