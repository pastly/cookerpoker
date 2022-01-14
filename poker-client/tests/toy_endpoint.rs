#![cfg(feature = "web-integration-tests")]
use std::env;
use reqwest::Client;
use poker_client::http;
use poker_messages::*;

#[tokio::test]
async fn available_and_accurate() {
    let url = format!(
        "http://{}:{}/api/foo",
        env::var("ROCKET_ADDRESS").unwrap(),
        env::var("ROCKET_PORT").unwrap());
    println!("{}", url);
    let c = Client::new();
    let action: Action = http::get_json(&c, url).await.unwrap();
    println!("{:?}", action);
    assert_eq!(action.seq, 1);
    let sd = match action.action {
        ActionEnum::SitDown(sd) => sd,
        _ => unreachable!(),
    };
    assert_eq!(sd.player_info.player_id, 10);
    assert_eq!(sd.player_info.name, "Mutt".to_string());
    assert_eq!(sd.player_info.monies, 100.into());
    assert_eq!(sd.player_info.seat, 0);
}
