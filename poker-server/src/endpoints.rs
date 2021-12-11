pub use super::AppError;
pub use crate::database::DbConn;
pub use crate::models;
pub use diesel::prelude::*;
pub use logic::{account::ApiKeyError, table::TableError};
pub use logic::{
    account::{Admin, User},
    table::GameTable,
};
pub use rocket::form::Form;
pub use rocket::response::Redirect;
pub use rocket_dyn_templates::{tera::Context, Template};

mod api;
pub mod accounts;
pub mod forms;
pub mod index;
pub mod logic;
pub mod tables;

pub fn get_all_endpoints() -> Vec<rocket::route::Route> {
    let mut v = tables::get_endpoints();
    v.append(&mut accounts::get_endpoints());
    v.append(&mut index::get_endpoints());
    v.append(&mut api::get_endpoints());
    v
}
