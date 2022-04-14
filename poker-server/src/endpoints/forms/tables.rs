use super::super::logic::table::TableState;
use poker_core::game::table::TableType;
use rocket::form::{self, FromFormField, ValueField};

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

impl<'r> FromFormField<'r> for TableType {
    fn from_value(field: form::ValueField<'r>) -> form::Result<'r, Self> {
        match field.value() {
            "Tournament" => TableType::Tournament,
            "Open" => TableType::Open,
            _ => Err(form::Error::validation("unknown table type").into())
        }
    }
}
