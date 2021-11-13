use rocket::http::Status;
use rocket::request::{Outcome, Request, FromRequest};
use crate::database::{DbConn, models::{SettledAccount, Account}};
use diesel::prelude::*;

async fn api_to_account(db: DbConn, key: String) -> Result<Account, ApiKeyError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let account = db.run(|conn| 
        accounts.filter(api_key.eq(key)).first(conn).map_err(|_|ApiKeyError::Invalid)
    );
    account.await
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

pub struct User(pub Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.headers().get_one("x-api-key") {
            Some(key) => key.to_string(),
            _ => return Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
        };

        match api_to_account(db, key).await {
            Ok(a) => Outcome::Success(User(a)),
            Err(_) => Outcome::Failure((Status::Forbidden, ApiKeyError::Invalid)),
        }
    }
}


pub struct Admin(pub Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.headers().get_one("x-api-key") {
            Some(key) => key.to_string(),
            _ => return Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
        };

        let account = match api_to_account(db, key).await {
            Ok(a) => a,
            Err(_) => return Outcome::Failure((Status::Forbidden, ApiKeyError::Invalid)),
        };

        if account.is_admin == 1 {
            Outcome::Success(Admin(account))
        } else {
            Outcome::Forward(())
        }
    }
}

pub async fn get_settled_balance(db: &DbConn, a: i32) -> i32 {
    use crate::database::schema::settled_accounts::dsl::{settled_accounts, account_id};
    db.run(move |conn| settled_accounts.filter(account_id.eq(a)).first(conn).map(|s: SettledAccount| s.get_monies()).expect("Account ID doesn't have settled balance")).await
}