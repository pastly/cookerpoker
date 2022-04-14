use super::logic::account::api_key_to_account;
use super::*;
use models::accounts::{Account, NewAccount};
use models::forms::LoginForm;
use rocket::form::Errors;
use rocket::http::{Cookie, CookieJar};
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        get_login,
        post_login,
        logout,
        get_id_monies,
        post_id_monies,
        monies_admin,
        monies_user,
        get_accounts,
        new_account
    ]
}

#[get("/login")]
async fn get_login(flash: Option<FlashMessage<'_>>) -> Template {
    let mut c = Context::new();
    if let Some(flash) = flash {
        let key = match flash.kind() {
            "success" | "error" => flash.kind(),
            _ => "message",
        };
        c.insert(key, flash.message());
    }
    Template::render("login", &c.into_json())
}

#[post("/login", data = "<form>")]
async fn post_login(
    jar: &CookieJar<'_>,
    db: DbConn,
    form: Result<Form<LoginForm>, Errors<'_>>,
) -> Result<Flash<Redirect>, Flash<Redirect>> {
    //let a = api_key_to_account(&db, &form.api_key).await?;
    let err_dest = Redirect::to("/login".to_string());
    let res = match form {
        Ok(form) => api_key_to_account(&db, &form.api_key).await,
        Err(e) => return Err(Flash::error(err_dest, e.to_string())),
    };
    match res {
        Ok(a) => {
            jar.add_private(Cookie::new("account", serde_json::to_string(&a).unwrap()));
            Ok(Flash::success(
                Redirect::to("/".to_string()),
                format!("Successfully logged in as {}.", a.account_name),
            ))
        }
        Err(e) => {
            let msg = match e {
                AppError::DbError(e) => match e.as_str() {
                    "NotFound" => "An account with that API key was not found.".to_string(),
                    _ => format!("Database error: {}", e),
                },
                AppError::ApiKeyError(e) => format!("API key error: {}", e),
                AppError::TableError(e) => format!("Table error: {}", e),
            };
            Err(Flash::error(err_dest, msg))
        }
    }
}

#[get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private(Cookie::named("account"));
    Redirect::to("/".to_string())
}

#[get("/monies/<id>")]
async fn get_id_monies(conn: DbConn, _a: Admin, id: i32) -> Result<Template, AppError> {
    //TODO Repleace id with request guard?
    let a = Account::find(&conn, id).await.map_err(AppError::from)?;
    let mut c = Context::new();
    c.insert("account_name", &a.account_name);
    c.insert("monies", &a.monies());
    Ok(Template::render("mod_settled", &c.into_json()))
}

#[post("/monies/<id>", data = "<change>")]
async fn post_id_monies(
    conn: DbConn,
    admin: Admin,
    id: i32,
    change: Form<forms::ModSettled>,
) -> Result<Redirect, AppError> {
    let target = Account::find(&conn, id).await?;
    target
        .mod_settled_balance(&admin, &conn, change.into_inner())
        .await?;
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
) -> Result<String, AppError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let na = NewAccount::from(f.into_inner());
    conn.run::<_, Result<String, AppError>>(|conn| {
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
