use super::super::logic::table::{TableState, TableType};

#[derive(FromForm)]
pub struct NewTable {
    pub table_name: String,
}

#[derive(FromForm)]
pub struct UpdateTableSettings {
    pub table_type: TableType,
    pub name: String,
    pub state: TableState,
    pub buy_in: i32,
    pub small_blind: i32,
}
