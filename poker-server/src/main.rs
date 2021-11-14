#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;

mod account;
mod database;

use database::DbConn;
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
    let mut v = Vec::new();
    v.append(&mut account::get_endpoints());
    v.append(&mut routes![index]);
    v
}

// TODO build a function to automatically delete the test admin in release mode.
