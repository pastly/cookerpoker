use super::*;
use schema::{accounts, money_log};

#[derive(Identifiable, Queryable, Insertable, Serialize, Debug)]
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
    pub made_by: i32,
}

impl NewMoneyLogEntry {
    pub fn new(me: &Account, target: &Account, form: forms::ModSettled) -> Self {
        NewMoneyLogEntry {
            account_id: target.id,
            monies: form.change,
            reason: form.reason,
            made_by: me.id,
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
