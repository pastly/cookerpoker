pub use crate::database::models::{Account, NewMoneyLogEntry, SettledAccount};
use crate::database::{DbConn, DbError};
use derive_more::Deref;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

pub mod endpoints;
pub mod forms;
pub use endpoints::get_endpoints;

async fn api_to_account(db: DbConn, key: String) -> Result<Account, ApiKeyError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let account = db.run(|conn| {
        accounts
            .filter(api_key.eq(key))
            .first(conn)
            .map_err(|_| ApiKeyError::Invalid)
    });
    account.await
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}

#[derive(Deref)]
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

#[derive(Deref)]
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

impl Account {
    pub async fn get_settled_balance(&self, db: &DbConn) -> Result<i32, DbError> {
        // Closure cannot refer to self apparently? Have to copy value out and let it take ownership of id
        self.get_settled_account(db)
            .await
            .map(|s: SettledAccount| s.get_monies())
    }

    pub async fn mod_settled_balance(
        &self,
        db: &DbConn,
        change: forms::ModSettled,
    ) -> Result<(), DbError> {
        // TODO technically supposed to be inside the transaction, but needs minor refactor
        // TODO record starting and ending balance?
        use crate::database::schema::money_log::dsl::money_log;
        let mut sb = self.get_settled_account(db).await?;
        sb += change.change;
        let nme = NewMoneyLogEntry::new(self, change);
        db.run(move |conn| {
            conn.transaction(|| {
                diesel::update(&sb).set(&sb).execute(conn)?;
                diesel::insert_into(money_log).values(nme).execute(conn)?;
                Ok(())
            })
            .map_err(|_: DbError| DbError::AccountNotFound)
        })
        .await
    }

    async fn get_settled_account(&self, db: &DbConn) -> Result<SettledAccount, DbError> {
        use crate::database::schema::settled_accounts::dsl::settled_accounts;
        let id = self.account_id;
        //TODO Return other DB errors
        db.run(move |conn| {
            settled_accounts
                .find(id)
                .first(conn)
                .map_err(|_| DbError::NoSettledBalance)
        })
        .await
    }

    pub async fn find(db: &DbConn, id: i32) -> Result<Account, DbError> {
        use crate::database::schema::accounts::dsl::accounts;
        //TODO Return other DB errors
        db.run(move |conn| {
            accounts
                .find(id)
                .first(conn)
                .map_err(|_| DbError::AccountNotFound)
        })
        .await
    }
}
