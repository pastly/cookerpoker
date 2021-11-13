pub mod schema;
pub mod models;
use rocket_sync_db_pools::{diesel, database};

#[database("sqlite")]
pub struct DbConn(diesel::SqliteConnection);