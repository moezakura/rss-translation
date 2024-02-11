use s3::error::ProvideErrorMetadata;
use sha2::{Digest, Sha256};

use super::provider::CacheProvider;
use aws_config::{meta::credentials::CredentialsProviderChain, Region};
use aws_sdk_s3::config::Builder;
use aws_sdk_s3 as s3;
use aws_sdk_s3::client::Client as S3Client;
use aws_smithy_types::byte_stream::ByteStream;
use aws_credential_types::Credentials;

use bytes::Bytes;
use std::{error::Error, future::Future, pin::Pin};

pub struct S3CacheProviderOptions {
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint_url: String,
    pub bucket_name: String,
}

pub struct S3CacheProvider {
    client: S3Client,
    bucket_name: String,
}

impl CacheProvider for S3CacheProvider {
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

        let s3_client = self.client.clone();
        let bucket_name = self.bucket_name.clone();
        let bucket_client = s3_client.get_object().bucket(bucket_name);
        Box::pin(async move {
            let object_data = bucket_client.key(file_name).send().await;
            if object_data.is_err() {
                // エラーがファイルが見つからない場合であればログにも残さない
                let object_data = object_data.unwrap_err();
                if object_data.code().as_deref() == Some("NoSuchKey"){
                    return Ok(None);
                }
                println!("failed to get object data: {:?}", object_data);
                return Ok(None);
            }
            let object_data = object_data.unwrap();

            let cached_data_bytes = object_data.body.collect().await?.into_bytes().to_vec();
            let cached_data = String::from_utf8(cached_data_bytes).unwrap();
            Ok(Some(cached_data))
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

        let value_bytes: Bytes = Bytes::from(value.into_bytes());
        let s3_client = self.client.clone();
        let bucket_name = self.bucket_name.clone();
        let bucket_client = s3_client.put_object().bucket(bucket_name);
        Box::pin(async move {
            let value_bytestream = ByteStream::from(value_bytes);
            bucket_client
                .key(file_name)
                .body(value_bytestream)
                .send()
                .await?;

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn CacheProvider> {
        Box::new(S3CacheProvider {
            client: self.client.clone(),
            bucket_name: self.bucket_name.clone(),
        })
    }
}

impl S3CacheProvider {
    pub fn new(options: S3CacheProviderOptions) -> Self {
        let access_key = options.access_key;
        let secret_key = options.secret_key;
        let credentials = Credentials::new(access_key, secret_key, None, None, "manual");

        let provider = CredentialsProviderChain::first_try("default", credentials);

        let sdk_config = Builder::new()
        .endpoint_url(options.endpoint_url)
        .credentials_provider(provider)
        .region(Region::new(options.region))
            .force_path_style(true)
            .build();

        let client = s3::Client::from_conf(sdk_config);
        S3CacheProvider {
            client,
            bucket_name: options.bucket_name,
        }
    }
}
