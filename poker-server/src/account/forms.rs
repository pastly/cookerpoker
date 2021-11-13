#[derive(FromForm)]
pub struct ModSettled {
    pub change: i32,
    pub reason: String,
}