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

#[derive(Debug, Responder)]
pub enum AppError {
    DbError(String),
    ApiKeyError(endpoints::ApiKeyError),
    TableError(endpoints::TableError),
}

impl From<endpoints::ApiKeyError> for AppError {
    fn from(e: endpoints::ApiKeyError) -> Self {
        Self::ApiKeyError(e)
    }
}

impl From<endpoints::TableError> for AppError {
    fn from(e: endpoints::TableError) -> Self {
        match e {
            endpoints::TableError::UnknownDbError(s) => Self::DbError(s),
            _ => Self::TableError(e),
        }
    }
}

impl std::convert::From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        // TODO do this for real
        AppError::DbError(e.to_string())
    }
}

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
