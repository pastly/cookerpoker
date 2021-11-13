#[derive(Queryable)]
pub struct SettledAccount {
    pub account_id: i32,
    monies: i32,
}

impl SettledAccount {
    pub fn get_monies(&self) -> i32 {
        self.monies
    }
}

#[derive(Queryable)]
pub struct Account {
    pub account_id: i32,
    pub account_name: String,
    pub api_token: String,
    pub is_admin: i16,
}
