use super::*;
use crate::database::models;
use crate::database::DbConn;
use diesel::prelude::*;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket_dyn_templates::{tera::to_value, tera::Context, Template};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        get_id_monies,
        post_id_monies,
        monies_admin,
        monies_user,
        get_accounts,
        new_account
    ]
}

#[get("/monies/<id>")]
async fn get_id_monies(conn: DbConn, _a: Admin, id: i32) -> Result<Template, DbError> {
    //TODO Repleace id with request guard?
    let a = Account::find(&conn, id)
        .await
        .map_err(|x| DbError::AccountNotFound(format!("{:?}", x)))?;
    let v = a
        .get_settled_balance(&conn)
        .await
        .map_err(|x| DbError::NoSettledBalance(format!("{:?}", x)))?;
    let mut c = Context::from_value(to_value(a).unwrap()).unwrap();
    c.insert("monies", &v);
    let c = c.into_json();
    Ok(Template::render("mod_settled", &c))
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

#[get("/accounts")]
async fn get_accounts(conn: DbConn, _a: Admin) -> Template {
    let accounts = Account::get_all(&conn).await.unwrap();
    let mut c = Context::new();
    c.insert("accounts", &accounts);
    Template::render("get_accounts", &c.into_json())
}

#[post("/accounts", data = "<f>")]
async fn new_account(
    conn: DbConn,
    _a: Admin,
    f: Form<forms::NewAccount>,
) -> Result<String, DbError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    use crate::database::schema::settled_accounts::dsl::settled_accounts;
    let na = models::NewAccount::from(f.into_inner());
    conn.run::<_, Result<String, DbError>>(|conn| {
        conn.transaction(|| {
            let api = na.api_key.clone();
            diesel::insert_into(accounts).values(na).execute(conn)?;
            let a = accounts
                .filter(api_key.eq(api.clone()))
                .first::<Account>(conn)?;
            info!("Created and returned account with id {}", a.account_id);
            let sb = models::SettledAccount::new(a.account_id);
            diesel::insert_into(settled_accounts)
                .values(sb)
                .execute(conn)?;
            Ok(api)
        })
    })
    .await
}
