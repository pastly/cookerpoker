#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

mod database;
mod account;

use database::DbConn;
use rocket_dyn_templates::{Template, tera::to_value, tera::Context};
use rocket::response::Redirect;
use rocket::form::Form;

#[get("/")]
fn index() ->&'static str {
    "This will eventually serve the poker client"
}

#[get("/monies/<id>")]
async fn get_id_monies(conn: DbConn, _a: account::Admin, id: i32) -> Template {
    //TODO Repleace id with request guard?
    let a = account::Account::find(&conn, id).await.expect("DBError unimplemented");
    let v = a.get_settled_balance(&conn).await.expect("DBError unimplemented");
    let mut c= Context::from_value(to_value(a).unwrap()).unwrap();
    c.insert("monies", &v);
    let c = c.into_json();
    Template::render("mod_settled", &c)
}

#[post("/monies/<id>", data = "<change>")]
async fn post_id_monies(conn: DbConn, _a: account::Admin, id: i32, change: Form<account::forms::ModSettled>) -> Redirect {
    let a = account::Account::find(&conn, id).await.expect("DBError unimplemented");
    a.mod_settled_balance(&conn, change.into_inner()).await.unwrap_or_else(|_| warn!("New money entry failed?"));
    Redirect::to(format!("/monies/{}", id))
}

#[get("/monies")]
async fn monies_admin(conn: DbConn, a: account::Admin) -> String {
    let v = a.get_settled_balance(&conn).await.expect("DbError unimplemented");
    format!("Welcome God-King {}. Your balance is {} pennies", a.account_name, v)
}

#[get("/monies", rank = 2)]
async fn monies_user(conn: DbConn, u: account::User) -> String {
    let v = u.get_settled_balance(&conn).await.expect("DbError unimplemented");
    format!("Welcome peasent. Your balance is {} pennies", v)
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount("/", routes![index, monies_admin, monies_user, get_id_monies, post_id_monies])
}
