#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::SqliteConnection;
use dotenv::dotenv;
use std::env;
const DATABASE_URL:&'static str ="test.db";

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
        SqliteConnection::establish(&DATABASE_URL)
        .expect(&format!("Error connecting to {}", DATABASE_URL))
}