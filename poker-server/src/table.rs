pub mod endpoints;
pub mod forms;

use crate::account::{api_to_account, ApiKeyError};
pub use crate::database::models::{Account, GameTable};
use crate::database::{DbConn, DbError};
use derive_more::{Deref, Display};
use diesel::prelude::*;
pub use endpoints::get_endpoints;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RenderedTable {
    pub id: i32,
    pub name: String,
    pub state: String,
}

impl From<GameTable> for RenderedTable {
    fn from(gt: GameTable) -> Self {
        Self {
            id: gt.id,
            name: gt.table_name,
            state: format!(
                "{}",
                TableState::try_from(gt.table_state).expect("Bad table state loaded from DB!")
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum TableState {
    GameNotReady,
    GameOpenNotStarted,
    ///Should only be used for 'open' tables
    GameOpenStarted,
    GameClosed,
    GameFinished,
}

impl TryFrom<i16> for TableState {
    type Error = TableError;
    fn try_from(f: i16) -> Result<Self, TableError> {
        match f {
            0 => Ok(Self::GameNotReady),
            1 => Ok(Self::GameOpenNotStarted),
            2 => Ok(Self::GameOpenStarted),
            3 => Ok(Self::GameClosed),
            4 => Ok(Self::GameFinished),
            _ => Err(TableError::InvalidTableState),
        }
    }
}

impl Into<i16> for TableState {
    fn into(self) -> i16 {
        match self {
            Self::GameNotReady => 0,
            Self::GameOpenNotStarted => 1,
            Self::GameOpenStarted => 2,
            Self::GameClosed => 3,
            Self::GameFinished => 4,
        }
    }
}

impl TableState {
    /// Helper function because dumb
    pub fn i(self) -> i16 {
        self.into()
    }
}

impl TableType {
    /// Helper function because dumb
    pub fn i(self) -> i16 {
        self.into()
    }
}

#[derive(Debug, FromFormField, Clone, Copy, Display)]
pub enum TableType {
    Tournament,
    Open,
}

impl TryFrom<i16> for TableType {
    type Error = TableError;
    fn try_from(f: i16) -> Result<Self, TableError> {
        match f {
            0 => Ok(Self::Tournament),
            1 => Ok(Self::Open),
            _ => Err(TableError::InvalidTableType),
        }
    }
}

impl Into<i16> for TableType {
    fn into(self) -> i16 {
        match self {
            Self::Tournament => 0,
            Self::Open => 1,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TableError {
    InvalidTableType,
    InvalidTableState,
}

#[derive(Deref)]
pub struct AdminOrTableOwner(pub Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminOrTableOwner {
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
            Outcome::Success(AdminOrTableOwner(account))
        } else {
            Outcome::Forward(())
        }

        //TODO check table owner
    }
}
