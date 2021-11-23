#![allow(unused_variables)]
#![allow(clippy::unused_unit)]
use super::logic::table::{AdminOrTableOwner, RenderedTable, TableState, TableType};
use super::*;
use crate::database::schema::game_tables;
use crate::models::tables::{GameTable, NewTable};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        get_tables,
        new_table,
        get_table_settings,
        update_table_settings,
        mod_table_settings,
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

#[get("/table/<id>", rank = 2)]
pub async fn get_table_settings(db: DbConn, _u: User, id: i32) -> Result<Template, DbError> {
    let t: GameTable = db
        .run(move |conn| game_tables::table.find(id).first(conn))
        .await?;
    let mut c = Context::new();
    c.insert("table", &RenderedTable::from(t));
    c.insert("is_disabled", "disabled");
    Ok(Template::render("table_settings", &c.into_json()))
}

#[get("/table/<id>")]
pub async fn mod_table_settings(
    db: DbConn,
    _u: AdminOrTableOwner,
    id: i32,
) -> Result<Template, DbError> {
    let t: GameTable = db
        .run(move |conn| game_tables::table.find(id).first(conn))
        .await?;
    let mut c = Context::new();
    c.insert("table", &RenderedTable::from(t));
    c.insert("is_disabled", "");
    c.insert("table_types", &TableType::get_all_as_str());
    c.insert("table_states", &TableState::get_all_as_str());
    Ok(Template::render("table_settings", &c.into_json()))
}

#[post("/table/<id>")]
pub async fn update_table_settings(
    db: DbConn,
    _a: AdminOrTableOwner,
    id: i32,
) -> Result<Redirect, DbError> {
    unimplemented!()
}
