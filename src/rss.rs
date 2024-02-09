use rss::Channel;
use std::error::Error;

#[derive(Clone)]
pub struct RssProvider;

impl RssProvider {
    pub fn new() -> RssProvider {
        RssProvider {}
    }

    pub async fn get_rss_feeds(&self, url: String) -> Result<Channel, Box<dyn Error>> {
        let content = reqwest::get(url).await?.bytes().await?;

        let channel = Channel::read_from(&content[..])?;

        Ok(channel)
    }
}
