use super::schema::{accounts, game_tables, money_log};
use crate::account::forms;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::dsl::{Eq, Select, Filter, Or};

#[derive(Identifiable, Queryable, Insertable, Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: i32,
    pub account_name: String,
    pub api_key: String,
    pub is_admin: i16,
    monies: i32,
}

impl Account {
    pub fn monies(&self) -> i32 {
        self.monies
    }
}

impl std::ops::AddAssign<i32> for Account {
    fn add_assign(&mut self, other: i32) {
        self.monies += other;
    }
}

#[derive(Insertable)]
#[table_name = "money_log"]
pub struct NewMoneyLogEntry {
    pub account_id: i32,
    pub reason: String,
    pub monies: i32,
}

impl NewMoneyLogEntry {
    pub fn new(a: &Account, form: forms::ModSettled) -> Self {
        NewMoneyLogEntry {
            account_id: a.id,
            monies: form.change,
            reason: form.reason,
        }
    }
}

#[derive(Insertable)]
#[table_name = "accounts"]
pub struct NewAccount {
    account_name: String,
    pub api_key: String,
    is_admin: i16,
}

impl From<forms::NewAccount> for NewAccount {
    fn from(f: forms::NewAccount) -> Self {
        let is_admin = if f.is_admin { 1i16 } else { 0i16 };
        NewAccount {
            account_name: f.account_name,
            is_admin,
            api_key: poker_core::util::random_string(42),
        }
    }
}

#[derive(Insertable)]
#[table_name = "game_tables"]
pub struct NewTable {
    table_name: String,
    table_type: i16,
}

impl NewTable {
    pub fn new(table_name: String, table_type: TableType) -> Self {
        NewTable {
            table_name,
            table_type: table_type.into(),
        }
    }
}

#[derive(Identifiable, Queryable, Serialize)]
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

pub type GameTableAllColumns = (game_tables::id, game_tables::table_owner, game_tables::table_type, game_tables::table_name, game_tables::table_state, game_tables::hand_num, game_tables::buy_in, game_tables::small_blind);

use crate::table::{TableError, TableType};
pub type SelectAllTables = Select<game_tables::table, GameTableAllColumns>;
pub type CheckOpenTableEq = Eq<game_tables::table_state, i16>;
pub type CheckTableOwner = Eq<game_tables::table_owner, i32>;
pub type OpenTableOr = Or<CheckOpenTableEq, CheckOpenTableEq>;
pub type OpenTableFilter = Filter<SelectAllTables, OpenTableOr>;
pub type OpenOrMyTables = Filter<SelectAllTables, Or<OpenTableOr, CheckTableOwner>>;
impl GameTable {
    pub fn table_type(&self) -> Result<TableType, TableError> {
        TableType::try_from(self.table_type)
    }

    pub fn all() -> SelectAllTables {
        game_tables::dsl::game_tables.select(game_tables::all_columns)
    }
    
    pub fn get_open() -> OpenTableFilter {
        use game_tables::dsl;
        use crate::table::TableState;
        
        Self::all().filter(dsl::table_state.eq(TableState::GameOpenStarted.i()).or(dsl::table_state.eq(TableState::GameOpenNotStarted.i())))
    }
    
    pub fn get_open_or_my_tables(table_owner: i32) -> OpenOrMyTables {
        use game_tables::dsl;
        Self::get_open().or_filter(dsl::table_owner.eq(table_owner))
    }
}
