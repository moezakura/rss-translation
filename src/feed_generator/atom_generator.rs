use std::str::FromStr;

use atom_syndication::Category as AtomCategory;
use atom_syndication::FixedDateTime;
use atom_syndication::Link as AtomLink;
use atom_syndication::Person as AtomPerson;
use atom_syndication::Text as AtomText;
use feed_rs::model::Feed;

use feed_rs::model::Link;

use super::feed_generator::FeedGenerator;

pub struct AtomGenerator;

impl FeedGenerator for AtomGenerator {
    fn generate_feed(&self, feed: Feed) -> String {
        // rs_feedのLinkをatom_syndication::Linkに変換
        fn to_links(links: Vec<Link>) -> Vec<AtomLink> {
            links
                .iter()
                .map(|link| {
                    let mut atom_link = AtomLink::default();
                    atom_link.set_href(link.href.clone());
                    if let Some(rel) = link.rel.clone() {
                        atom_link.set_rel(rel);
                    }
                    if let Some(mime_type) = link.media_type.clone() {
                        atom_link.set_mime_type(mime_type);
                    }
                    atom_link.set_title(link.title.clone());
                    if let Some(length) = link.length.clone() {
                        let length_string = length.to_string();
                        atom_link.set_length(length_string);
                    }
                    atom_link
                })
                .collect()
        }

        // フィードのタイトル
        let feed_title: AtomText = match feed.title {
            Some(title) => AtomText::plain(title.content),
            None => AtomText::plain("Untitled"),
        };
        // フィードのアイコン
        let feed_icon: Option<String> = match feed.icon {
            Some(icon) => Some(icon.uri),
            None => None,
        };
        // フィードのID
        let feed_id = feed.id.clone();
        // フィードの更新日時
        let feed_updated: Option<String> = match feed.updated {
            Some(updated) => Some(updated.to_rfc3339()),
            None => None,
        };
        // フィードのリンク
        let feed_links: Vec<AtomLink> = to_links(feed.links);
        // subtitle
        let feed_subtitle: AtomText = match feed.description {
            Some(description) => AtomText::plain(description.content),
            None => AtomText::plain(""),
        };
        // フィードのauthor
        let feed_author: Vec<AtomPerson> = feed
            .authors
            .iter()
            .map(|author| {
                let mut atom_author = AtomPerson::default();
                atom_author.set_name(author.name.clone());
                atom_author.set_email(author.email.clone());
                atom_author
            })
            .collect();
        // フィードのカテゴリ
        let feed_category: Vec<AtomCategory> = feed
            .categories
            .iter()
            .map(|category| {
                let mut atom_category = AtomCategory::default();
                atom_category.set_term(category.term.clone());
                if let Some(scheme) = category.scheme.clone() {
                    atom_category.set_scheme(scheme);
                }
                atom_category
            })
            .collect();

        // フィードのロゴ
        let feed_logo: Option<String> = match feed.logo {
            Some(logo) => Some(logo.uri),
            None => None,
        };

        let mut atom = atom_syndication::Feed::default();
        atom.set_title(feed_title);
        atom.set_icon(feed_icon);
        atom.set_id(feed_id);
        if feed_updated.is_some() {
            let updated_at_str = feed_updated.unwrap();
            let updated_at = FixedDateTime::from_str(&updated_at_str);
            if updated_at.is_ok() {
                atom.set_updated(updated_at.unwrap());
            }
        }
        atom.set_links(feed_links);
        atom.set_subtitle(feed_subtitle);
        atom.set_authors(feed_author);
        atom.set_categories(feed_category);
        atom.set_logo(feed_logo);

        for item in feed.entries.iter() {
            // アイテムのタイトル
            let item_title: AtomText = match item.clone().title {
                Some(title) => AtomText::plain(title.content),
                None => AtomText::plain("Untitle"),
            };
            // アイテムのID
            let item_id = item.id.clone();
            // アイテムの更新日時
            let item_updated: Option<String> = match item.updated {
                Some(updated) => Some(updated.to_rfc3339()),
                None => None,
            };
            // アイテムのリンク
            let item_link = to_links(item.links.clone());
            // アイテムの説明
            let item_description: AtomText = match item.clone().summary {
                Some(description) => {
                    // descriptionのcontent_typeに合わせて作る
                    if description.content_type == mime::TEXT_HTML {
                        AtomText::html(description.content)
                    } else {
                        AtomText::plain(description.content)
                    }
                }
                None => AtomText::plain(""),
            };
            // アイテムのカテゴリ
            let item_category: Vec<AtomCategory> = item
                .categories
                .iter()
                .map(|category| {
                    let mut atom_category = AtomCategory::default();
                    atom_category.set_term(category.term.clone());
                    if let Some(scheme) = category.scheme.clone() {
                        atom_category.set_scheme(scheme);
                    }
                    atom_category
                })
                .collect();
            // アイテムの著者
            let item_author: Vec<AtomPerson> = item
                .authors
                .iter()
                .map(|author| {
                    let mut atom_author = AtomPerson::default();
                    atom_author.set_name(author.name.clone());
                    atom_author.set_email(author.email.clone());
                    atom_author
                })
                .collect();
            //アイテムの公開日時
            let item_published: Option<String> = match item.published {
                Some(published) => Some(published.to_rfc3339()),
                None => None,
            };

            let mut atom_entry = atom_syndication::Entry::default();
            atom_entry.set_title(item_title);
            atom_entry.set_id(item_id);
            if item_updated.is_some() {
                let updated_at_str = item_updated.unwrap();
                let updated_at = FixedDateTime::from_str(&updated_at_str);
                if updated_at.is_ok() {
                    atom_entry.set_updated(updated_at.unwrap());
                }
            }
            atom_entry.set_links(item_link);
            atom_entry.set_summary(item_description);
            atom_entry.set_categories(item_category);
            atom_entry.set_authors(item_author);
            if item_published.is_some() {
                let published_at_str = item_published.unwrap();
                let published_at = FixedDateTime::from_str(&published_at_str);
                if published_at.is_ok() {
                    atom_entry.set_published(published_at.unwrap());
                }
            }

            atom.entries.push(atom_entry);
        }

        atom.to_string()
    }

    fn content_type(&self) -> String {
        "application/atom+xml".to_string()
    }
}

impl AtomGenerator {
    pub fn new() -> AtomGenerator {
        AtomGenerator {}
    }
}
