#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

pub mod database;
mod endpoints;
pub mod models;
pub use database::{schema, DbConn};
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", FileServer::from("./static"))
        .mount("/", get_all_endpoints())
}

fn get_all_endpoints() -> Vec<rocket::route::Route> {
    endpoints::get_all_endpoints()
}

// TODO build a function to automatically delete the test admin in release mode.
