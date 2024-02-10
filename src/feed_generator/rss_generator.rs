use feed_rs::model::Feed;
use rss::Category as RssCategory;

use super::feed_generator::FeedGenerator;

pub struct RssGenerator;

impl FeedGenerator for RssGenerator {
    fn generate_feed(&self, channel: Feed) -> String {
        // チャンネルのタイトル
        let channel_title: String = match channel.title {
            Some(title) => title.content,
            None => "Untitled".to_string(),
        };
        // チャンネルのリンク
        let channel_link = if channel.links.len() > 0 {
            channel.links[0].href.clone()
        } else {
            "".to_string()
        };

        // チャンネルの説明
        let channel_description: String = match channel.description {
            Some(description) => description.content,
            None => "".to_string(),
        };
        // チャンネルの最終更新日時
        let channel_last_build_date: Option<String> = match channel.updated {
            Some(updated) => Some(updated.to_rfc2822()),
            None => None,
        };

        let mut rss = rss::ChannelBuilder::default()
            .title(channel_title)
            .link(channel_link)
            .description(channel_description)
            .last_build_date(channel_last_build_date)
            .build();

        for item in channel.entries {
            //  アイテムのタイトル
            let item_title: Option<String> = match item.title {
                Some(title) => Some(title.content),
                None => None,
            };
            // アイテムのリンク
            let item_link = if item.links.len() > 0 {
                Some(item.links[0].href.clone())
            } else {
                None
            };
            // アイテムの説明
            let item_description = match item.summary {
                Some(description) => Some(description.content),
                None => None,
            };
            //アイテムの更新日時
            let item_pub_date: Option<String> = match item.updated {
                Some(updated) => Some(updated.to_rfc2822()),
                None => None,
            };
            //アイテムのカテゴリ
            let item_category: Vec<RssCategory> = item
                .categories
                .iter()
                .map(|category| {
                    let mut rss_category = RssCategory::default();
                    rss_category.set_name(category.term.clone());
                    if let Some(scheme) = category.scheme.clone() {
                        rss_category.set_domain(scheme);
                    }

                    rss_category
                })
                .collect();
            // 著者
            let item_author = match item.authors.len() {
                0 => None,
                _ => {
                    let joined_authors = item
                        .authors
                        .iter()
                        .map(|author| author.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ");
                    Some(joined_authors)
                }
            };

            let rss_item = rss::ItemBuilder::default()
                .title(item_title)
                .link(item_link)
                .description(item_description)
                .pub_date(item_pub_date)
                .categories(item_category)
                .author(item_author)
                .build();

            rss.items.push(rss_item);
        }

        rss.to_string()
    }

    fn content_type(&self) -> String {
        "application/rss+xml".to_string()
    }
}

impl RssGenerator {
    pub fn new() -> RssGenerator {
        RssGenerator {}
    }
}
