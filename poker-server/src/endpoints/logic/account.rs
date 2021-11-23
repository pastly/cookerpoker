use super::*;
pub use crate::models::accounts::{Account, NewMoneyLogEntry};
use crate::AppError;
use rocket::http::{CookieJar, Status};

///TODO I think there is a better way to do this. Return the dsl directly
pub async fn cookie_to_account(
    db: &DbConn,
    cookies: &'_ CookieJar<'_>,
) -> Result<Account, AppError> {
    use crate::database::schema::accounts::dsl::{accounts, api_key};
    let key = match cookies.get("api-key") {
        Some(key) => key.value().to_string(),
        None => return Err(ApiKeyError::Missing(()).into()),
    };
    let account = db.run(|conn| {
        accounts
            .filter(api_key.eq(key))
            .first(conn)
            .map_err(AppError::from)
    });
    account.await
}

#[derive(Debug, Responder)]
pub enum ApiKeyError {
    #[response(status = 400)]
    Missing(()),
    #[response(status = 404)]
    Invalid(()),
}

#[derive(Deref)]
pub struct User(Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let account = match cookie_to_account(&db, req.cookies()).await {
            Ok(a) => a,
            Err(e) => return Outcome::Failure((Status::Forbidden, e)),
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
        let db = req.guard::<DbConn>().await.unwrap();

        let account = match cookie_to_account(&db, req.cookies()).await {
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
