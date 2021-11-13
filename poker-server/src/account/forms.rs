#[derive(FromForm)]
pub struct ModSettled {
    pub change: i32,
    pub reason: String,
}

#[derive(FromForm)]
pub struct NewAccount {
    pub name: String,
    pub is_admin: bool,
}