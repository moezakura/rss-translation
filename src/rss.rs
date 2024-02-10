use std::error::Error;
use feed_rs::{model::Feed, parser};

#[derive(Clone)]
pub struct RssProvider;

impl RssProvider {
    pub fn new() -> RssProvider {
        RssProvider {}
    }

    pub async fn get_rss_feeds(&self, url: String) -> Result<Feed, Box<dyn Error>> {
        let content = reqwest::get(url).await?.bytes().await?;

        let feed = parser::parse(&content[..])?;

        Ok(feed)
    }
}
