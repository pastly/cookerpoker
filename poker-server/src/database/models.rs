#[derive(Queryable)]
pub struct SettledAccount {
    pub account_id: i32,
    pub monies: i32,
}

#[derive(Queryable)]
pub struct Account {
    pub account_id: i32,
    pub account_name: String,
    pub api_token: String,
    pub is_admin: i16,
}
