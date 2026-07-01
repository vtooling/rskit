//! HTTP client helpers built on `reqwest`. Requires the `http` feature.

use std::time::Duration;

use anyhow::Result;
use reqwest::{Client, IntoUrl, Method, Response};
use serde::Serialize;

/// Shared default HTTP client (30s timeout).
pub static HTTP_CLIENT: std::sync::LazyLock<Client> = std::sync::LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("create http client error")
});

/// Build a custom client.
pub fn build_client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(30)).build()?)
}

/// Send a JSON request. `data` is serialized as the JSON body.
pub async fn request<U, D>(method: Method, url: U, data: &D) -> Result<Response>
where
    U: IntoUrl,
    D: Serialize + ?Sized,
{
    Ok(HTTP_CLIENT.request(method, url).json(data).send().await?)
}

/// GET a URL, returning the raw response.
pub async fn get<U: IntoUrl>(url: U) -> Result<Response> {
    Ok(HTTP_CLIENT.get(url).send().await?)
}

/// POST a JSON body.
pub async fn post_json<U: IntoUrl, D: Serialize + ?Sized>(url: U, data: &D) -> Result<Response> {
    Ok(HTTP_CLIENT.post(url).json(data).send().await?)
}

/// GET a URL and decode the JSON response body into `T`.
pub async fn get_json<U: IntoUrl, T: serde::de::DeserializeOwned>(url: U) -> Result<T> {
    let resp = get(url).await?;
    Ok(resp.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_client_is_built() {
        let _ = &*HTTP_CLIENT;
    }

    #[test]
    fn build_client_succeeds() {
        assert!(build_client().is_ok());
    }
}
