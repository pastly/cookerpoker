use super::*;
use crate::database::DbConn;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket_dyn_templates::{tera::to_value, tera::Context, Template};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![get_id_monies, post_id_monies, monies_admin, monies_user]
}

#[get("/monies/<id>")]
async fn get_id_monies(conn: DbConn, _a: Admin, id: i32) -> Template {
    //TODO Repleace id with request guard?
    let a = Account::find(&conn, id)
        .await
        .expect("DBError unimplemented");
    let v = a
        .get_settled_balance(&conn)
        .await
        .expect("DBError unimplemented");
    let mut c = Context::from_value(to_value(a).unwrap()).unwrap();
    c.insert("monies", &v);
    let c = c.into_json();
    Template::render("mod_settled", &c)
}

#[post("/monies/<id>", data = "<change>")]
async fn post_id_monies(
    conn: DbConn,
    _a: Admin,
    id: i32,
    change: Form<forms::ModSettled>,
) -> Redirect {
    let a = Account::find(&conn, id)
        .await
        .expect("DBError unimplemented");
    a.mod_settled_balance(&conn, change.into_inner())
        .await
        .unwrap_or_else(|_| warn!("New money entry failed?"));
    Redirect::to(format!("/monies/{}", id))
}

#[get("/monies")]
async fn monies_admin(conn: DbConn, a: Admin) -> String {
    let v = a
        .get_settled_balance(&conn)
        .await
        .expect("DbError unimplemented");
    format!(
        "Welcome God-King {}. Your balance is {} pennies",
        a.account_name, v
    )
}

#[get("/monies", rank = 2)]
async fn monies_user(conn: DbConn, u: User) -> String {
    let v = u
        .get_settled_balance(&conn)
        .await
        .expect("DbError unimplemented");
    format!("Welcome peasent. Your balance is {} pennies", v)
}
