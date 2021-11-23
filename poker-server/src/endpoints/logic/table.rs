use super::account::cookie_to_account;
use super::*;
pub use crate::models::{accounts::Account, tables::GameTable};
use crate::AppError;
use rocket::http::Status;
use schema::game_tables;

#[derive(Debug, Clone, Serialize)]
pub struct RenderedTable {
    pub id: i32,
    pub name: String,
    pub state: String,
    pub buy_in: i32,
    pub small_blind: i32,
    pub table_type: String,
    // TODO figure out how to show owner name
    pub owner: i32,
}

impl From<GameTable> for RenderedTable {
    fn from(gt: GameTable) -> Self {
        Self {
            id: gt.id,
            name: gt.table_name,
            state: TableState::try_from(gt.table_state)
                .expect("Bad table state loaded from DB!")
                .to_string(),
            buy_in: gt.buy_in,
            small_blind: gt.small_blind,
            table_type: TableState::try_from(gt.table_type)
                .expect("Bad table type loaded from DB!")
                .to_string(),
            owner: gt.table_owner,
        }
    }
}

#[derive(Debug, Clone, Copy, Display, FromFormField)]
pub enum TableState {
    NotReady,
    OpenNotStarted,
    ///Should only be used for 'open' tables
    OpenStarted,
    Closed,
    Finished,
}

impl TryFrom<i16> for TableState {
    type Error = TableError;
    fn try_from(f: i16) -> Result<Self, TableError> {
        match f {
            0 => Ok(Self::NotReady),
            1 => Ok(Self::OpenNotStarted),
            2 => Ok(Self::OpenStarted),
            3 => Ok(Self::Closed),
            4 => Ok(Self::Finished),
            _ => Err(TableError::InvalidTableState(TableState::get_error())),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i16> for TableState {
    fn into(self) -> i16 {
        match self {
            Self::NotReady => 0,
            Self::OpenNotStarted => 1,
            Self::OpenStarted => 2,
            Self::Closed => 3,
            Self::Finished => 4,
        }
    }
}

impl TableState {
    /// Helper function because dumb
    pub fn i(self) -> i16 {
        self.into()
    }

    pub const fn get_all_as_slice() -> [&'static str; 5] {
        [
            "NotReady",
            "OpenNotStarted",
            "OpenStarted",
            "Closed",
            "Finished",
        ]
    }

    pub const fn get_error() -> &'static str {
        // TODO figure out how to do this from slice
        "Invalid TableState. Valid values are: NotReady, OpenNotStarted, OpenStarted, Closed, Finished"
    }
}

impl TableType {
    /// Helper function because dumb
    pub fn i(self) -> i16 {
        self.into()
    }
    pub const fn get_all_as_slice() -> [&'static str; 2] {
        ["Tournament", "Open"]
    }
    pub const fn get_error() -> &'static str {
        // TODO figure out how to do this from slice
        "Invalid TableType. Valid values are: Tournament, Open"
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
            _ => Err(TableError::InvalidTableType(TableType::get_error())),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<i16> for TableType {
    fn into(self) -> i16 {
        match self {
            Self::Tournament => 0,
            Self::Open => 1,
        }
    }
}

#[derive(Debug, Responder)]
pub enum TableError {
    #[response(status = 400)]
    InvalidTableType(&'static str),
    #[response(status = 400)]
    InvalidTableState(&'static str),
    #[response(status = 404)]
    TableNotFound(()),
    #[response(status = 400)]
    TableNameAlreadyTaken(&'static str),
    #[response(status = 500)]
    UnknownDbError(String),
}

impl std::convert::From<diesel::result::Error> for TableError {
    fn from(e: diesel::result::Error) -> Self {
        use diesel::result::{DatabaseErrorKind, Error};
        match e {
            Error::NotFound => TableError::TableNotFound(()),
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                TableError::TableNameAlreadyTaken("Table name already in use")
            }
            _ => TableError::UnknownDbError(e.to_string()),
        }
    }
}

#[derive(Deref)]
pub struct AdminOrTableOwner(pub Account);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminOrTableOwner {
    type Error = AppError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = req.guard::<DbConn>().await.unwrap();

        let account = match cookie_to_account(&db, req.cookies()).await {
            Ok(a) => a,
            Err(e) => return Outcome::Failure((Status::Forbidden, e)),
        };

        if account.is_admin == 1 {
            Outcome::Success(AdminOrTableOwner(account))
        } else {
            let t_id: i32 = req
                .param(1)
                .expect("No table id somehow?")
                .expect("Couldn't parse table ID into i32 somehow?");
            let t: Result<GameTable, TableError> = db
                .run(move |conn| {
                    game_tables::table
                        .find(t_id)
                        .first(conn)
                        .map_err(|_| TableError::TableNotFound(()))
                })
                .await;
            let t = match t {
                Ok(x) => x,
                Err(e) => return Outcome::Failure((Status::NotFound, AppError::from(e))),
            };
            if t.table_owner == account.id {
                Outcome::Success(AdminOrTableOwner(account))
            } else {
                Outcome::Forward(())
            }
        }

        //TODO check table owner
    }
}
