use super::*;
use crate::database::models;
use crate::database::DbConn;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket_dyn_templates::{tera::Context, Template};

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
    let mut c = Context::new();
    c.insert("account_name", &a.account_name);
    c.insert("monies", &a.monies());
    Ok(Template::render("mod_settled", &c.into_json()))
}

#[post("/monies/<id>", data = "<change>")]
async fn post_id_monies(
    conn: DbConn,
    _a: Admin,
    id: i32,
    change: Form<forms::ModSettled>,
) -> Result<Redirect, DbError> {
    let a = Account::find(&conn, id).await?;
    a.mod_settled_balance(&conn, change.into_inner()).await?;
    Ok(Redirect::to(format!("/monies/{}", id)))
}

#[get("/monies")]
async fn monies_admin(a: Admin) -> String {
    format!(
        "Welcome God-King {}. Your balance is {} pennies",
        a.account_name,
        a.monies()
    )
}

#[get("/monies", rank = 2)]
async fn monies_user(u: User) -> String {
    format!("Welcome peasent. Your balance is {} pennies", u.monies())
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
    let na = models::NewAccount::from(f.into_inner());
    conn.run::<_, Result<String, DbError>>(|conn| {
        let api = na.api_key.clone();
        diesel::insert_into(accounts).values(na).execute(conn)?;
        // Dirty read because Diesel doesn't support SQLite's RETURNING yet
        let a = accounts
            .filter(api_key.eq(api.clone()))
            .first::<Account>(conn)?;
        info!("Created and returned account with id {}", a.id);
        Ok(api)
    })
    .await
}
