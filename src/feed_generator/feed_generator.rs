use feed_rs::model::Feed;

pub trait FeedGenerator {
    fn generate_feed(&self, feed: Feed) -> String;
    fn content_type(&self) -> String;
}


