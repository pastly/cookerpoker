#![cfg(feature = "web-integration-tests")]
use poker_client::http;
use poker_core::util::random_string;
use reqwest::{redirect::Policy, Client};
use std::collections::HashMap;
use std::env;
use std::process::Command;

const ADMIN_API_KEY: &str = "KcMj5tZOssjajWhGeUeVByvckEjucthPVOmjygiBhX";
const USER_API_KEY: &str = "Og7Ixf1bRkPtXfKR4tgHyurSwwA0lkkQ4uYJIPuVNs";
const ADMIN_TABLE_ID: i64 = 1;
const USER_TABLE_ID: i64 = 2;

fn sql(q: &str) -> String {
    let output = Command::new("sqlite3")
        .arg(env::var("DB_PATH").unwrap())
        .arg(q)
        .output()
        .expect("Unable to execute sqlite3 without error")
        .stdout;
    String::from_utf8(output).unwrap()
}

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

fn post_data() -> HashMap<&'static str, &'static str> {
    let mut data = HashMap::new();
    data.insert("table_type", "Tournament");
    data.insert("name", "foo");
    data.insert("state", "NotReady");
    data.insert("buy_in", "500");
    data.insert("small_blind", "5");
    data
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
    // should probably be 403
    assert_eq!(resp.status(), 404);
}

/// Dual-purpose test and helper function.
///
/// Create a new table as the current logged-in user via the HTTP API and assert it shows up on
/// the table listing page.
///
/// Also connect to the database directly and retrieve its ID.
///
/// Return the ID and name of the new table.
async fn create_table(c: &Client) -> (i64, String) {
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

    let output = sql(&format!(
        "SELECT id FROM game_tables WHERE table_name = '{name}'"
    ));
    let id = output.trim().parse::<i64>().unwrap();
    (id, name)
}

#[tokio::test]
async fn create_table_admin() {
    let c = admin_client(Default::default()).await;
    create_table(&c).await;
}

#[tokio::test]
async fn create_table_nonadmin() {
    let c = user_client(Default::default()).await;
    create_table(&c).await;
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
    // should probably be 403
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn view_tables_anon() {
    let c = anon_client(Default::default()).await;
    let url = table_url_prefix();
    let resp = http::get(&c, url).await.unwrap();
    // should probably be 403
    assert_eq!(resp.status(), 404);
}

/// Can see your own table, but no one elses unless it's an open table.
#[ignore = "todo. See #47"]
#[tokio::test]
async fn view_tables_nonadmin() {
    let c = user_client(Default::default()).await;
    let url = table_url_prefix();
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(text.contains("table-notmatt"));
    assert!(!text.contains("table-matt"));
    // todo. Need to also check that an open table that isn't owned by this person is visible.
    assert!(false);
}

/// Can see all tables
#[ignore = "Known failure. See #47"]
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
async fn get_table_settings_anon() {
    let c = anon_client(Default::default()).await;

    let url = table_url_prefix() + &format!("/{ADMIN_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 403);

    let url = table_url_prefix() + &format!("/{USER_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 403);
}

// Regular users should be able to see the details page for their own tables, but not others
#[ignore = "Known failure. See #47"]
#[tokio::test]
async fn get_table_settings_nonadmin() {
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
async fn get_table_settings_admin() {
    let c = admin_client(Default::default()).await;

    let url = table_url_prefix() + &format!("/{USER_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);

    let url = table_url_prefix() + &format!("/{ADMIN_TABLE_ID}");
    let resp = http::get(&c, url).await.unwrap();
    assert_eq!(resp.status(), 200);
}

/// Anonymous person can't edit any table
#[tokio::test]
async fn post_table_settings_anon() {
    // create a table as some user. it isn't important who, because we're anon
    let id = {
        let c = user_client(Default::default()).await;
        let (id, _) = create_table(&c).await;
        id
    };
    let c = anon_client(Default::default()).await;
    let url = table_url_prefix() + &format!("/{id}");
    let resp = http::post_form(&c, url, &post_data()).await.unwrap();
    assert_eq!(resp.status(), 403);
}

/// Logged in non-admin can edit a table of their own, but not one owned by someone else
#[ignore = "No one can edit tables right now b/c of bug. See #47"]
#[tokio::test]
async fn post_table_settings_nonadmin() {
    // Can't edit someone else's table
    let admin_tbl_id = {
        let c = admin_client(Default::default()).await;
        let (id, _) = create_table(&c).await;
        id
    };
    let c = user_client(Default::default()).await;
    let url = table_url_prefix() + &format!("/{admin_tbl_id}");
    let resp = http::post_form(&c, url, &post_data()).await.unwrap();
    // should probably be 403
    assert_eq!(resp.status(), 404);

    // Can edit own table
    let user_tbl_id = {
        let c = user_client(Default::default()).await;
        let (id, _) = create_table(&c).await;
        id
    };
    let url = table_url_prefix() + &format!("/{user_tbl_id}");
    // table_type=Tournament&name=foo&state=NotReady&buy_in=500&small_blind=5"
    let resp = http::post_form(&c, url, &post_data()).await.unwrap();
    println!("{:?}", resp);
    assert!(false);
}

/// Admins can edit their tables and someone elses.
#[ignore = "No one can edit tables right no b/c of a bug. See #47"]
#[tokio::test]
async fn post_table_settings_admin() {
    // start impl based on _nonadmin version above
    assert!(false);
}
