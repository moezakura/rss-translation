use sqlx::AnyConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::provider::CacheProvider;

use std::{error::Error, future::Future, pin::Pin};

pub struct SqlCacheProviderOptions {
    pub connection_pool: AnyConnection,
}

pub struct SqlCacheProvider {
    connection_pool: Arc<Mutex<AnyConnection>>,
}

impl CacheProvider for SqlCacheProvider {
    fn get(
        &self,
        key: String,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, Box<dyn Error + '_>>> + Send + 'static>>
    {
        let connection = Arc::clone(&self.connection_pool);
        Box::pin(async move {
            let mut connection = connection.lock().await;

            let exec_query =
                sqlx::query_as("SELECT 'translated_title' FROM rss_cache WHERE raw_title = ?")
                    .bind(key);

            let row: (String,) = exec_query.fetch_one(&mut *connection).await?;

            let title = row.0.clone();

            Ok(Some(title))
        })
    }

    fn set(
        &self,
        key: String,
        value: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + '_>>> + Send + 'static>> {
        let connection = Arc::clone(&self.connection_pool);

        Box::pin(async move {
            let mut connection = connection.lock().await;

            let exec_query =
                sqlx::query("INSERT INTO rss_cache (raw_title, translated_title) VALUES (?, ?)")
                    .bind(key)
                    .bind(value);

            exec_query.execute(&mut *connection).await?;

            Ok(())
        })
    }

    fn clone_box(&self) -> Box<dyn CacheProvider> {
        Box::new(SqlCacheProvider {
            connection_pool: self.connection_pool.clone(),
        })
    }
}

impl SqlCacheProvider {
    pub fn new(options: SqlCacheProviderOptions) -> Self {
        SqlCacheProvider {
            connection_pool: Arc::new(Mutex::new(options.connection_pool)),
        }
    }
}
