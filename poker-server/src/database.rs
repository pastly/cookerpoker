pub mod schema;
pub mod models;
use rocket_sync_db_pools::{diesel, database};

#[database("sqlite")]
pub struct DbConn(diesel::SqliteConnection);

#[derive(Debug)]
pub enum DbError{
    NoSettledBalance,
    AccountNotFound,
    Unknown
}

impl std::convert::From<diesel::result::Error> for DbError {
    fn from(other: diesel::result::Error) -> Self {
        // TODO do this for real
        DbError::Unknown
    }
}