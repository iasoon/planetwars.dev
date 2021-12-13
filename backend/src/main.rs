#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod db;
mod routes;
mod schema;

#[database("postgresql_database")]
pub struct DbConn(diesel::PgConnection);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                routes::users::register,
                routes::users::login,
                routes::users::current_user,
            ],
        )
        .attach(DbConn::fairing())
}
