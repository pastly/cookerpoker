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

#[cfg(test)]
mod tests {
    use super::*;
    use poker_messages::{Action, ActionEnum};

    #[tokio::test]
    async fn foo() {
        let c = Client::new();
        let resp = get_json::<Action, _>(&c, "http://127.0.0.1:8000/api/foo")
            .await
            .unwrap();
        assert_eq!(resp.seq, 1);
        assert!(matches!(resp.action, ActionEnum::SitDown(_)));
    }
}
