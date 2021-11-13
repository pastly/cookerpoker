#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

mod database;
mod account;

use database::DbConn;
use rocket_dyn_templates::{Template, tera::to_value, tera::Context};
use rocket::response::Redirect;
use rocket::form::Form;

#[get("/")]
fn index() ->&'static str {
    "This will eventually serve the poker client"
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", get_all_endpoints())
}

fn get_all_endpoints() -> Vec<rocket::route::Route> {
    let mut v = Vec::new();
    v.append(&mut account::get_endpoints());
    v
}