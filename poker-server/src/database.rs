pub mod models;
pub mod schema;
use rocket_sync_db_pools::{database, diesel};

#[database("sqlite")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug)]
pub enum DbError {
    NoSettledBalance,
    AccountNotFound,
    Unknown,
}

impl std::convert::From<diesel::result::Error> for DbError {
    fn from(_other: diesel::result::Error) -> Self {
        // TODO do this for real
        DbError::Unknown
    }
}
