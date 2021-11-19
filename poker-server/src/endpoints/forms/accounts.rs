#[derive(FromForm)]
pub struct ModSettled {
    pub change: i32,
    pub reason: String,
}

#[derive(FromForm)]
pub struct NewAccount {
    pub account_name: String,
    pub is_admin: bool,
}
