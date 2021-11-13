use rocket::http::Status;
use rocket::request::{self, Outcome, Request, FromRequest};
use crate::database::{DbConn, models::{SettledAccount, Account}};
use diesel::prelude::*;

pub struct ApiKey<'r>(&'r str);

async fn api_to_account(db: DbConn, key: String) -> Result<Account, ApiKeyError> {
    use crate::database::schema::settled_accounts::dsl::settled_accounts;
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let account = db.run(move |conn| 
        accounts.filter(api_key.eq(key)).first(conn).map_err(|_|ApiKeyError::Invalid)
    );
    account.await
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(key: &str) -> bool {
            // TODO
            key.len()>10
        }

        match req.headers().get_one("x-api-key") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

pub struct User(pub Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.headers().get_one("x-api-key") {
            Some(key) => key.to_string(),
            _ => return Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
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
        /// Returns true if `key` is a valid API key string.
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.headers().get_one("x-api-key") {
            Some(key) => key.to_string(),
            _ => return Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
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