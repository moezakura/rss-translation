use feed_rs::{model::Feed, parser};
use std::error::Error;

#[derive(Clone)]
pub struct RssProvider {
    client: reqwest::Client,
}

impl RssProvider {
    pub fn new() -> RssProvider {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36";
        let client = reqwest::ClientBuilder::new()
            .user_agent(user_agent.to_string())
            .build()
            .unwrap();

        RssProvider { client }
    }

    pub async fn get_rss_feeds(&self, url: String) -> Result<Feed, Box<dyn Error>> {
        let client = self.client.clone();
        let content = client
            .get(url.clone())
            .header(reqwest::header::DNT, "1")
            .send()
            .await?
            .bytes()
            .await?;

        let feed = parser::parse(&content[..])?;

        Ok(feed)
    }
}
