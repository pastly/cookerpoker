#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod endpoints;
pub mod models;
pub mod database;
pub use database::{DbConn, schema};
use rocket_dyn_templates::Template;

#[get("/")]
fn index() -> &'static str {
    "This will eventually serve the poker client"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", get_all_endpoints())
}

fn get_all_endpoints() -> Vec<rocket::route::Route> {
    let mut v = routes![index];
    v.append(&mut endpoints::get_all_endpoints());
    v
}

// TODO build a function to automatically delete the test admin in release mode.
