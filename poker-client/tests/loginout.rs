#![cfg(feature = "web-integration-tests")]
use poker_client::http;
use reqwest::{redirect, Client};
use std::collections::HashMap;
use std::env;

const ADMIN_API_KEY: &str = "KcMj5tZOssjajWhGeUeVByvckEjucthPVOmjygiBhX";
const ADMIN_NAME: &str = "matt";

fn url_prefix() -> String {
    format!(
        "http://{}:{}",
        env::var("ROCKET_ADDRESS").unwrap(),
        env::var("ROCKET_PORT").unwrap(),
    )
}

#[tokio::test]
async fn root_shows_login_link() {
    let root = url_prefix();
    let c = Client::new();
    let resp = http::get(&c, root).await.unwrap().text().await.unwrap();
    assert!(!resp.contains(ADMIN_NAME));
    assert!(resp.contains("<a href=/login>"));
}

#[tokio::test]
async fn login_works() {
    // make login request
    let login_url = url_prefix() + "/login";
    let c = Client::builder()
        .cookie_store(true)
        .redirect(redirect::Policy::none())
        .build()
        .unwrap();
    let mut data = HashMap::new();
    data.insert("api_key", ADMIN_API_KEY);
    let resp = http::post_form(&c, login_url, &data).await.unwrap();
    // should contain Set-Cookie for account
    assert!(resp.cookies().find(|c| c.name() == "account").is_some());
    // See if a request for the root page now shows our name and a logout link.
    let resp = http::get(&c, url_prefix())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(resp.contains(ADMIN_NAME));
    assert!(resp.contains("<a href=/logout"));
}

#[tokio::test]
async fn logout_works() {
    // make login request
    let login_url = url_prefix() + "/login";
    let c = Client::builder()
        .cookie_store(true)
        .redirect(redirect::Policy::none())
        .build()
        .unwrap();
    let mut data = HashMap::new();
    data.insert("api_key", ADMIN_API_KEY);
    http::post_form(&c, login_url, &data).await.unwrap();
    // now try logging out
    let logout_url = url_prefix() + "/logout";
    let resp = http::get(&c, logout_url).await.unwrap();
    // should have been told to erase cookie
    let cookie = resp.cookies().find(|c| c.name() == "account").unwrap();
    assert_eq!(cookie.value(), "");
    // and another request should result in the not-logged-in home page
    let resp_text = http::get(&c, url_prefix())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!resp_text.contains(ADMIN_NAME));
    assert!(resp_text.contains("<a href=/login"));
}
