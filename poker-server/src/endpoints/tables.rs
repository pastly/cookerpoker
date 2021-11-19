#![allow(unused_variables)]
#![allow(clippy::unused_unit)]
use super::logic::{
    account::User,
    table::{AdminOrTableOwner, RenderedTable},
};
use super::*;
use crate::database::schema::game_tables;
use crate::models::tables::{GameTable, NewTable};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        get_tables,
        new_table,
        get_table_settings,
        update_table_settings,
    ]
}

// TODO GameTableError
#[get("/tables")]
pub async fn get_tables(db: DbConn, u: User) -> Result<Template, DbError> {
    let uid = u.id;
    let tables: Vec<RenderedTable> = db
        .run(move |conn| GameTable::get_open_or_my_tables(uid).get_results::<GameTable>(conn))
        .await?
        .into_iter()
        .map(RenderedTable::from)
        .collect();
    let mut c = Context::new();
    c.insert("tables", &tables);
    Ok(Template::render("list_tables", &c.into_json()))
}

#[post("/table", data = "<nt>")]
pub async fn new_table(
    db: DbConn,
    u: User,
    nt: Form<forms::NewTable>,
) -> Result<Redirect, DbError> {
    let ntf = nt.into_inner();
    let nt = NewTable::new(u.id, ntf.table_name);
    db.run(move |conn| {
        diesel::insert_into(game_tables::table)
            .values(&nt)
            .execute(conn)
    })
    .await?;
    Ok(Redirect::to("/tables"))
}

#[get("/table/<id>")]
pub async fn get_table_settings(db: DbConn, _u: User, id: i32) -> () {}

#[post("/table/<id>")]
pub async fn update_table_settings(db: DbConn, _a: AdminOrTableOwner, id: i32) -> () {}
