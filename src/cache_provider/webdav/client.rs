use base64::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use url::Url;
use std::error::Error;

#[derive(Clone)]
pub struct WebDavClient {
    http_client: reqwest::Client,
    base_url: Url,
}

impl WebDavClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        let auth_header_value = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}:{}", username, password))
        );
        let auth_header_value = HeaderValue::from_str(&auth_header_value).unwrap();
        let mut auth_header = HeaderMap::new();
        auth_header.append("Authorization", auth_header_value);

        let client = reqwest::Client::builder()
            .default_headers(auth_header)
            .build()
            .unwrap();

        let base_url = Url::parse(base_url).unwrap();

        WebDavClient {
            http_client: client,
            base_url: base_url.to_owned(),
        }
    }

    pub async fn get(&self, path: String) -> Result<Option<String>, Box<dyn Error>> {
        let target_url = self.base_url.join(&path)?;
        let response = self.http_client.get(target_url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }
        let response_body = response.text().await?;
        Ok(
            Some(response_body)
        )
    }

    pub async fn set(&self, url: String, value: String) -> Result<(), Box<dyn Error>> {
        let target_url = self.base_url.join(&url)?;

        let response = self
            .http_client
            .put(target_url)
            .body(value)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to set cache".into());
        }

        Ok(())
    }
}
