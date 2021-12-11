use super::*;
use poker_messages::{Action, ActionEnum, SitDown};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![
        foo,
    ]
}

#[get("/api/foo")]
async fn foo() -> String {
    let a = Action::new(
        1,
        ActionEnum::SitDown(SitDown::new(10, "Mutt".to_string(), 100, 0)),
    );
    serde_json::to_string(&a).unwrap()
}
