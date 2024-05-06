use sqlx::{Any, Pool};
use std::{fmt, sync::Arc};
use tokio::sync::Mutex;

use super::provider::CacheProvider;

use std::{error::Error, future::Future, pin::Pin};

pub struct SqlCacheProviderOptions {
    pub connection_pool: Pool<Any>,
}

pub struct SqlCacheProvider {
    connection_pool: Arc<Mutex<Pool<Any>>>,
}

#[derive(Debug)]
struct GetError {
    message: String,
}

impl fmt::Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GetError: {}", self.message)
    }
}
impl Error for GetError {}

impl CacheProvider for SqlCacheProvider {
    fn get(
        &self,
        key: String,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, Box<dyn Error + '_>>> + Send + 'static>>
    {
        let connection_pool = Arc::clone(&self.connection_pool);

        let sql_thread = tokio::spawn(async move {
            let connection = connection_pool.lock().await;
            let mut pool = connection.acquire().await?;

            let result: Result<(Vec<u8>,), sqlx::error::Error> = sqlx::query_as(
                "SELECT `translated_title` FROM rss_cache WHERE raw_title = ? LIMIT 1",
            )
            .bind(key)
            .fetch_one(&mut *pool)
            .await;

            result
        });

        Box::pin(async move {
            let result = sql_thread.await.unwrap();

            match result {
                Ok(row) => {
                    let title = row.0.clone();
                    // vec<u8> -> String
                    let title = String::from_utf8(title).unwrap();

                    Ok(Some(title))
                }
                Err(sqlx::Error::RowNotFound) => Ok(None),
                Err(e) => Err(SqlCacheProvider::create_get_error(e.to_string())),
            }
        })
    }

    fn set(
        &self,
        key: String,
        value: String,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + '_>>> + Send + 'static>> {
        let connection_pool = Arc::clone(&self.connection_pool);

        Box::pin(async move {
            let connection = connection_pool.lock().await;
            let mut pool = connection.acquire().await?;

            let exec_query =
                sqlx::query("INSERT INTO rss_cache (raw_title, translated_title) VALUES (?, ?)")
                    .bind(key)
                    .bind(value);

            exec_query.execute(&mut *pool).await?;

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

    pub fn create_get_error(message: String) -> Box<dyn Error> {
        Box::new(GetError { message: message })
    }
}
