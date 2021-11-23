pub mod schema;
use rocket_sync_db_pools::{database, diesel};

#[database("sqlite")]
pub struct DbConn(diesel::SqliteConnection);
