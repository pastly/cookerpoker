pub(crate) use reqwest::Client;
use reqwest::{IntoUrl, Result};
use serde::Deserialize;

/// Make a GET request to the given URL, expect a JSON response, parse the JSON response into the
/// appropriate type, and return it.  Returns reqwest::Error if anything fails.
pub async fn get_json<T: for<'de> Deserialize<'de>, U: IntoUrl>(c: &Client, url: U) -> Result<T> {
    c.get(url).send().await?.json::<T>().await
}

pub async fn get<U: IntoUrl>(c: &Client, url: U) -> Result<String> {
    c.get(url).send().await?.text().await
}

#[cfg(feature = "testing")]
pub fn get_sync<U: IntoUrl>(c: &Client, url: U) -> Result<String> {
    let future = async move { get(c, url).await };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
