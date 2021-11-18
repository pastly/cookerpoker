use super::*;

#[derive(FromForm)]
pub struct NewTable {
    pub table_name: String,
    pub table_type: TableType,
}
