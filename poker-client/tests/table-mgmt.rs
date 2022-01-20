#![cfg(feature = "web-integration-tests")]
use poker_client::http;
use poker_core::util::random_string;
use reqwest::{redirect::Policy, Client};
use std::collections::HashMap;
use std::env;

const ADMIN_API_KEY: &str = "KcMj5tZOssjajWhGeUeVByvckEjucthPVOmjygiBhX";
const USER_API_KEY: &str = "Og7Ixf1bRkPtXfKR4tgHyurSwwA0lkkQ4uYJIPuVNs";
const ADMIN_TABLE_ID: i32 = 1;
const USER_TABLE_ID: i32 = 2;

fn url_prefix() -> String {
    format!(
        "http://{}:{}",
        env::var("ROCKET_ADDRESS").unwrap(),
        env::var("ROCKET_PORT").unwrap(),
    )
}

fn table_url_prefix() -> String {
    url_prefix() + "/tables"
}

async fn anon_client(redir_policy: Policy) -> Client {
    let c = Client::builder()
        .cookie_store(true)
        .redirect(redir_policy)
        .build()
        .unwrap();
    c
}

async fn admin_client(redir_policy: Policy) -> Client {
    let c = anon_client(redir_policy).await;
    let mut data = HashMap::new();
    data.insert("api_key", ADMIN_API_KEY);
    http::post_form(&c, url_prefix() + "/login", &data)
        .await
        .unwrap();
    c
}

async fn user_client(redir_policy: Policy) -> Client {
    let c = anon_client(redir_policy).await;
    let mut data = HashMap::new();
    data.insert("api_key", USER_API_KEY);
    http::post_form(&c, url_prefix() + "/login", &data)
        .await
        .unwrap();
    c
}

#[tokio::test]
async fn get_table_page() {
    let c = admin_client(Default::default()).await;
    let resp = http::get(&c, table_url_prefix()).await.unwrap();
    // make a bunch of spot tests to assure ourselves that this is the page that lists existing
    // tables and the form for adding a new one.
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("<table"));
    assert!(text.contains("Table Settings"));
    assert!(text.contains("<form"));
    assert!(text.contains("Table Name"));
}

#[tokio::test]
async fn get_table_page_anon() {
    let c = anon_client(Default::default()).await;
    let resp = http::get(&c, table_url_prefix()).await.unwrap();
    assert_eq!(resp.status(), 404);
}

async fn create_table(c: Client) {
    let name = random_string(10);
    let mut data = HashMap::new();
    data.insert("table_name", &name);
    let resp = http::post_form(&c, table_url_prefix(), &data)
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.url().path(), "/tables");
    let text = resp.text().await.unwrap();
    assert!(text.contains(&name));
}

#[tokio::test]
async fn create_table_admin() {
    let c = admin_client(Default::default()).await;
    create_table(c).await;
}

#[tokio::test]
async fn create_table_nonadmin() {
    let c = user_client(Default::default()).await;
    create_table(c).await;
}

#[tokio::test]
async fn create_table_anon() {
    let c = anon_client(Default::default()).await;
    let name = random_string(10);
    let mut data = HashMap::new();
    data.insert("table_name", &name);
    let resp = http::post_form(&c, table_url_prefix(), &data)
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn view_tables_anon() {
    let c = anon_client(Default::default()).await;
    let url = table_url_prefix();
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 404);
}

/// Can see your own table, but no one elses
#[tokio::test]
async fn view_tables_nonadmin() {
    let c = user_client(Default::default()).await;
    let url = table_url_prefix();
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("table-notmatt"));
    assert!(!text.contains("table-matt"));
}

/// Can see all tables
#[tokio::test]
async fn view_tables_admin() {
    let c = admin_client(Default::default()).await;
    let url = table_url_prefix();
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("table-notmatt"));
    assert!(text.contains("table-matt"));
}

// Anonymous people should not be able to view table details
#[tokio::test]
async fn table_settings_anon() {
    let c = anon_client(Default::default()).await;

    let url = table_url_prefix() + &format!("/{ADMIN_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 403);

    let url = table_url_prefix() + &format!("/{USER_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 403);
}

// Regular users should should be able to see the details page for their own tables, but not others
#[tokio::test]
async fn table_settings_nonadmin() {
    let c = user_client(Default::default()).await;

    let url = table_url_prefix() + &format!("/{USER_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);

    let url = table_url_prefix() + &format!("/{ADMIN_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 403);
}

// Admins should should be able to see the details page for all tables
#[tokio::test]
async fn table_settings_admin() {
    let c = admin_client(Default::default()).await;

    let url = table_url_prefix() + &format!("/{USER_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);

    let url = table_url_prefix() + &format!("/{ADMIN_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);
}
