pub mod models;
pub mod schema;
use rocket::response::Responder;
use rocket_sync_db_pools::{database, diesel};

#[database("sqlite")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug, Responder)]
pub enum DbError {
    #[response(status = 500)]
    NoSettledBalance(String),
    #[response(status = 400)]
    AccountNotFound(String),
    #[response(status = 500)]
    Unknown(String),
}

impl std::convert::From<diesel::result::Error> for DbError {
    fn from(other: diesel::result::Error) -> Self {
        // TODO do this for real
        DbError::Unknown(format!("{:?}", other))
    }
}
