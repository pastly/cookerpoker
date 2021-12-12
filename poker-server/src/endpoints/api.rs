use poker_messages::{Action, ActionEnum, PlayerInfo, SitDown};

pub fn get_endpoints() -> Vec<rocket::route::Route> {
    routes![foo,]
}

#[get("/api/foo")]
async fn foo() -> String {
    let a = Action {
        seq: 1,
        action: ActionEnum::SitDown(SitDown::new(PlayerInfo::new(
            10,
            "Mutt".to_string(),
            100,
            0,
        ))),
    };
    serde_json::to_string(&a).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;

    fn client() -> Client {
        use super::super::super::rocket as myrocket;
        Client::tracked(myrocket()).expect("valid rocket client")
    }

    #[test]
    fn foo() {
        let c = client();
        let req = c.get("/api/foo");
        let resp = req.dispatch().into_json::<Action>().unwrap();
        println!("{:?}", resp);
        assert_eq!(resp.seq, 1);
        assert!(matches!(resp.action, ActionEnum::SitDown(_)));
    }
}
