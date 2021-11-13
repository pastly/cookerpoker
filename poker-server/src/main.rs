#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

mod database;
mod account;

use database::DbConn;

#[get("/")]
fn index() ->&'static str {
    "This will eventually serve the poker client"
}

#[get("/monies")]
async fn monies(conn: DbConn) -> String {
    "Not implemented".into()
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(DbConn::fairing())
        .mount("/", routes![index, monies])
}
