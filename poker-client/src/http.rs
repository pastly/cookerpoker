pub(crate) use reqwest::Client;
use reqwest::{IntoUrl, Result};
use serde::Deserialize;

/// Make a GET request to the given URL, expect a JSON response, parse the JSON response into the
/// appropriate type, and return it.  Returns reqwest::Error if anything fails.
pub(crate) async fn get_json<T: for<'de> Deserialize<'de>, U: IntoUrl>(
    c: &Client,
    url: U,
) -> Result<T> {
    c.get(url).send().await?.json::<T>().await
}
