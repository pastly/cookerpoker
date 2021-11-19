pub use crate::database::{DbConn, DbError};
pub use crate::models;
pub use diesel::prelude::*;
pub use rocket::form::Form;
pub use rocket::response::Redirect;
pub use rocket_dyn_templates::{tera::Context, Template};

pub mod accounts;
pub mod forms;
pub mod logic;
pub mod tables;

pub fn get_all_endpoints() -> Vec<rocket::route::Route> {
    let mut v = tables::get_endpoints();
    v.append(&mut accounts::get_endpoints());
    v
}
