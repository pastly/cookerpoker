use super::*;
use crate::account::{Account, Admin, User};
use crate::database::models::GameTable;
use crate::database::{DbConn, DbError};
use derive_more::Deref;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket_dyn_templates::{tera::to_value, tera::Context, Template};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        get_tables,
        new_table,
        get_table_settings,
        update_table_settings,
    ]
}

// TODO GameTableError
#[get("/table")]
pub async fn get_tables(db: DbConn, u: User) -> Result<Template, DbError> {
    let uid = u.id;
    let tables: Vec<RenderedTable> = db.run(move |conn| GameTable::get_open_or_my_tables(uid).get_results::<GameTable>(conn)).await.map_err(|x| DbError::from(x))?.into_iter().map(|x| RenderedTable::from(x)).collect();
    let mut c = Context::new();
    c.insert("tables", &tables);
    Ok(Template::render("list_tables", &c.into_json()))
}

#[post("/table")]
pub async fn new_table(db: DbConn, _a: Admin) -> () {}

#[get("/table/<id>")]
pub async fn get_table_settings(db: DbConn, _u: User, id: i32) -> () {}

#[post("/table/<id>")]
pub async fn update_table_settings(db: DbConn, _a: AdminOrTableOwner, id: i32) -> () {}
