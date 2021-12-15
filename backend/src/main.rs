#[macro_use]
extern crate rocket;
extern crate mozaic4_backend;

#[launch]
fn launch() -> rocket::Rocket<rocket::Build> {
    mozaic4_backend::rocket()
}
