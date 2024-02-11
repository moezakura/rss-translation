use sha2::{Digest, Sha256};

use super::provider::CacheProvider;

mod client;
use client::WebDavClient;
use std::{error::Error, future::Future, pin::Pin};

pub struct WebDavCacheProviderOptions {
    pub user_id: String,
    pub user_password: String,
    pub webdav_url: String,
}

pub struct WebDavCacheProvider {
    client: WebDavClient,
}

impl CacheProvider for WebDavCacheProvider {
    fn get(
        &self,
        key: String,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, Box<dyn Error + '_>>> + Send + 'static>>
    {
        // keyをsha256でハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hashed_key = hasher.finalize();
        let file_name = format!("{:x}", hashed_key);

        let webdav_client = self.client.clone();
        Box::pin(async move {
            let cached_data = webdav_client.get(file_name);
            match cached_data.await {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        })
    }

    fn set(
        &self,
        key: String,
        value: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + '_>>> + Send + 'static>> {
        // keyをsha256でハッシュ化
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hashed_key = hasher.finalize();
        let file_name = format!("{:x}", hashed_key);

        let webdav_client = self.client.clone();
        Box::pin(async move {
            let result = webdav_client.set(file_name, value);
            match result.await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        })
    }

    fn clone_box(&self) -> Box<dyn CacheProvider> {
        Box::new(WebDavCacheProvider {
            client: self.client.clone(),
        })
    }
}

impl WebDavCacheProvider {
    pub fn new(options: WebDavCacheProviderOptions) -> Self {
        WebDavCacheProvider {
            client: WebDavClient::new(
                &options.webdav_url,
                &options.user_id,
                &options.user_password,
            ),
        }
    }
}
