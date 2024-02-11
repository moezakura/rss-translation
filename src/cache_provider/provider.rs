use std::{error::Error, future::Future, pin::Pin};

pub trait CacheProvider: Send + Sync {
    fn get(&self, key: String) -> Pin<Box<dyn Future<Output = Result<Option<String>, Box<dyn Error+ '_>>> + Send+ 'static>>;
    fn set(&self, key: String, value: String) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error+ '_>>> + Send+ 'static>>;

    fn clone_box(&self) -> Box<dyn CacheProvider>;
}

impl Clone for Box<dyn CacheProvider> {
    fn clone(&self) -> Box<dyn CacheProvider> {
        self.clone_box()
    }
}
