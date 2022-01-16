pub(crate) use reqwest::Client;
use reqwest::{IntoUrl, Response, Result};
use serde::{Deserialize, Serialize};

/// Make a GET request to the given URL, expect a JSON response, parse the JSON response into the
/// appropriate type, and return it.  Returns reqwest::Error if anything fails.
pub async fn get_json<T: for<'de> Deserialize<'de>, U: IntoUrl>(c: &Client, url: U) -> Result<T> {
    c.get(url).send().await?.json::<T>().await
}

pub async fn get<U: IntoUrl>(c: &Client, url: U) -> Result<Response> {
    c.get(url).send().await
}

pub async fn post_form<U: IntoUrl, T: Serialize + ?Sized>(
    c: &Client,
    url: U,
    data: &T,
) -> Result<Response> {
    c.post(url).form(data).send().await
}
