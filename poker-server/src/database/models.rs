use super::schema::{money_log, settled_accounts};
use crate::account::forms::ModSettled;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Serialize, Deserialize, AsChangeset)]
#[primary_key(account_id)]
pub struct SettledAccount {
    pub account_id: i32,
    monies: i32,
}

impl SettledAccount {
    pub fn get_monies(&self) -> i32 {
        self.monies
    }
}

impl std::ops::AddAssign<i32> for SettledAccount {
    fn add_assign(&mut self, other: i32) {
        self.monies += other;
    }
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Account {
    pub account_id: i32,
    pub account_name: String,
    pub api_token: String,
    pub is_admin: i16,
}

#[derive(Insertable)]
#[table_name = "money_log"]
pub struct NewMoneyLogEntry {
    account_id: i32,
    reason: String,
    monies: i32,
}

impl NewMoneyLogEntry {
    pub fn new(a: &Account, form: ModSettled) -> Self {
        NewMoneyLogEntry {
            account_id: a.account_id,
            monies: form.change,
            reason: form.reason,
        }
    }
}
