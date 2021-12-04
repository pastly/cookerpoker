use super::*;
use crate::endpoints::logic::table::{TableError, TableState, TableType};
use schema::game_tables;

#[derive(Insertable)]
#[table_name = "game_tables"]
pub struct NewTable {
    table_owner: i32,
    table_name: String,
}

impl NewTable {
    pub fn new(table_owner: i32, table_name: String) -> Self {
        NewTable {
            table_owner,
            table_name,
        }
    }
}

#[derive(Identifiable, Queryable, Serialize, AsChangeset)]
pub struct GameTable {
    pub id: i32,
    pub table_owner: i32,
    pub table_type: i16,
    pub table_name: String,
    pub table_state: i16,
    pub hand_num: i32,
    pub buy_in: i32,
    pub small_blind: i32,
}

pub type GameTableAllColumns = (
    game_tables::id,
    game_tables::table_owner,
    game_tables::table_type,
    game_tables::table_name,
    game_tables::table_state,
    game_tables::hand_num,
    game_tables::buy_in,
    game_tables::small_blind,
);

pub type SelectAllTables = Select<game_tables::table, GameTableAllColumns>;
pub type CheckOpenTableEq = Eq<game_tables::table_state, i16>;
pub type CheckTableOwner = Eq<game_tables::table_owner, i32>;
pub type OpenTableOr = Or<CheckOpenTableEq, CheckOpenTableEq>;
pub type OpenTableFilter = Filter<SelectAllTables, OpenTableOr>;
pub type OpenOrMyTables = Filter<SelectAllTables, Or<OpenTableOr, CheckTableOwner>>;
impl GameTable {
    pub fn table_type(&self) -> Result<TableType, TableError> {
        let tt = TableType::from(self.table_type);
        match tt {
            TableType::Invalid => Err(TableError::InvalidTableType(TableType::get_error())),
            _ => Ok(tt),
        }
    }

    pub fn all() -> SelectAllTables {
        game_tables::dsl::game_tables.select(game_tables::all_columns)
    }

    pub fn get_open() -> OpenTableFilter {
        use game_tables::dsl;

        Self::all().filter(
            dsl::table_state
                .eq(TableState::OpenStarted.i())
                .or(dsl::table_state.eq(TableState::OpenNotStarted.i())),
        )
    }

    pub fn get_open_or_my_tables(table_owner: i32) -> OpenOrMyTables {
        use game_tables::dsl;
        Self::get_open().or_filter(dsl::table_owner.eq(table_owner))
    }

    pub fn update_settings(
        &mut self,
        form: crate::endpoints::forms::tables::UpdateTableSettings,
    ) -> Result<(), TableError> {
        // TODO Sanity Check. i.e. fail on thousand dollar buy ins.
        if self.table_state == TableState::NotReady.i() {
            self.table_name = form.name;
            self.table_type = form.table_type;
            self.table_state = form.state.into();
            self.buy_in = form.buy_in;
            self.small_blind = form.small_blind;
            Ok(())
        } else {
            Err(TableError::CannotModifyStartedGames(
                "Game is in progress. It cannot be modified",
            ))
        }
    }
}
