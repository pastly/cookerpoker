use super::*;
pub use crate::models::accounts::{Account, NewMoneyLogEntry};
use crate::AppError;
use rocket::form;
use rocket::http::{CookieJar, Status};

///TODO I think there is a better way to do this. Return the dsl directly
pub async fn cookie_to_account(cookies: &'_ CookieJar<'_>) -> Result<Account, AppError> {
    match cookies.get_private("account") {
        Some(key) => match serde_json::from_str(key.value()) {
            Ok(a) => Ok(a),
            Err(_) => Err(ApiKeyError::Invalid("Unable to parse account cookie").into()),
        },
        None => Err(ApiKeyError::Missing("account cookie is missing").into()),
    }
}

pub async fn api_key_to_account(db: &DbConn, key: &ApiKey) -> Result<Account, AppError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let k = key.0.clone();
    let account = db.run(|conn| {
        accounts
            .filter(api_key.eq(k))
            .first(conn)
            .map_err(AppError::from)
    });
    account.await
}

#[derive(Debug)]
pub struct ApiKey(String);

#[rocket::async_trait]
impl<'r> form::FromFormField<'r> for ApiKey {
    fn from_value(field: form::ValueField<'r>) -> form::Result<'r, Self> {
        if field.value.chars().count() != 42 {
            return Err(form::Error::validation("incorrect length").into());
        }
        Ok(Self(field.value.to_string()))
    }
}

#[derive(Debug, Responder, derive_more::Display)]
pub enum ApiKeyError {
    #[response(status = 400)]
    Missing(&'static str),
    #[response(status = 404)]
    Invalid(&'static str),
}

impl std::error::Error for ApiKeyError {}

#[derive(Deref)]
pub struct User(Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let account = match cookie_to_account(req.cookies()).await {
            Ok(a) => a,
            Err(e) => {
                return match e {
                    AppError::DbError(_) => Outcome::Failure((Status::InternalServerError, e)),
                    AppError::ApiKeyError(_) => Outcome::Forward(()),
                    AppError::TableError(_) => Outcome::Failure((Status::InternalServerError, e)),
                }
            }
        };

        Outcome::Success(User(account))
    }
}

#[derive(Deref)]
pub struct Admin(Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let account = match cookie_to_account(req.cookies()).await {
            Ok(a) => a,
            Err(e) => return Outcome::Failure((Status::Forbidden, e)),
        };

        if account.is_admin == 1 {
            Outcome::Success(Admin(account))
        } else {
            Outcome::Forward(())
        }
    }
}

impl Account {
    pub async fn mod_settled_balance(
        &self,
        admin: &Admin,
        db: &DbConn,
        change: forms::ModSettled,
    ) -> Result<i32, AppError> {
        // TODO record starting and ending balance?
        use crate::database::schema::accounts::dsl::{accounts, monies};
        use crate::database::schema::money_log::dsl::money_log;
        let nme = NewMoneyLogEntry::new(admin, self, change);
        db.run(move |conn| {
            conn.transaction::<i32, AppError, _>(|| {
                // Reload self to verify current balance inside transaction
                let a: Account = accounts.find(nme.account_id).first(conn)?;
                let ov = a.monies();
                let nv = ov + nme.monies;
                diesel::update(&a).set(monies.eq(nv)).execute(conn)?;
                diesel::insert_into(money_log).values(nme).execute(conn)?;
                Ok(nv)
            })
        })
        .await
    }

    pub async fn find(db: &DbConn, id: i32) -> Result<Account, AppError> {
        use crate::database::schema::accounts::dsl::accounts;
        //TODO Return other DB errors
        db.run(move |conn| accounts.find(id).first(conn).map_err(AppError::from))
            .await
    }

    pub async fn get_all(db: &DbConn) -> Result<Vec<Account>, AppError> {
        use crate::database::schema::accounts::dsl::accounts;
        db.run(|conn| accounts.load(conn).map_err(AppError::from))
            .await
    }
}
