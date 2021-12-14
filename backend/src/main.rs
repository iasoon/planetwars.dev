#[macro_use]
extern crate rocket;
extern crate mozaic4_backend;

#[launch]
fn launch() -> _ {
    mozaic4_backend::rocket()
}
