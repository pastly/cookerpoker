use super::*;
pub use crate::models::accounts::{Account, NewMoneyLogEntry};
use rocket::http::Status;

///TODO I think there is a better way to do this. Return the dsl directly
pub async fn api_to_account(db: DbConn, key: String) -> Result<Account, ApiKeyError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let account = db.run(|conn| {
        accounts
            .filter(api_key.eq(key))
            .first(conn)
            .map_err(ApiKeyError::from)
    });
    account.await
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    DbError(diesel::result::Error),
}

impl From<diesel::result::Error> for ApiKeyError {
    fn from(e: diesel::result::Error) -> Self {
        use diesel::result::Error::*;
        match e {
            NotFound => ApiKeyError::Invalid,
            _ => ApiKeyError::DbError(e),
        }
    }
}

#[derive(Deref)]
pub struct User(Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.cookies().get("api-key") {
            Some(key) => key.value().to_string(),
            _ => return Outcome::Forward(()),
        };

        match api_to_account(db, key).await {
            Ok(a) => Outcome::Success(User(a)),
            Err(e) => Outcome::Failure((Status::Forbidden, e)),
        }
    }
}

#[derive(Deref)]
pub struct Admin(Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let key = match req.cookies().get("api-key") {
            Some(key) => key.value().to_string(),
            _ => return Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
        };

        let account = match api_to_account(db, key).await {
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
        db: &DbConn,
        change: forms::ModSettled,
    ) -> Result<i32, DbError> {
        // TODO record starting and ending balance?
        use crate::database::schema::accounts::dsl::{accounts, monies};
        use crate::database::schema::money_log::dsl::money_log;
        let nme = NewMoneyLogEntry::new(self, change);
        db.run(move |conn| {
            conn.transaction::<i32, DbError, _>(|| {
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

    pub async fn find(db: &DbConn, id: i32) -> Result<Account, DbError> {
        use crate::database::schema::accounts::dsl::accounts;
        //TODO Return other DB errors
        db.run(move |conn| {
            accounts
                .find(id)
                .first(conn)
                .map_err(|x| DbError::AccountNotFound(format!("{:?}", x)))
        })
        .await
    }

    pub async fn get_all(db: &DbConn) -> Result<Vec<Account>, DbError> {
        use crate::database::schema::accounts::dsl::accounts;
        db.run(|conn| accounts.load(conn).map_err(DbError::from))
            .await
    }
}
