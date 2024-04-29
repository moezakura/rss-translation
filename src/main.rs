use actix_web::HttpRequest;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use feed_rs::model::{FeedType, Text};
use serde::Deserialize;

use rss_trans::rss as rtr;
use rss_trans::translate;
mod feed_generator;
use feed_generator::atom_generator::AtomGenerator;
use feed_generator::feed_generator::FeedGenerator;
use feed_generator::rss_generator::RssGenerator;
mod cache_provider;
use cache_provider::provider::CacheProvider;
use cache_provider::s3::{S3CacheProvider, S3CacheProviderOptions};
use cache_provider::webdav::{WebDavCacheProvider, WebDavCacheProviderOptions};

struct AppState {
    rss_provider: rtr::RssProvider,
    translate_provider: translate::TranslateProvider,
    translated_cache_provider: Option<Box<dyn CacheProvider>>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("index")
}

#[derive(Deserialize)]
struct RssReqQuery {
    url: String,
    to: Option<String>,
}

#[derive(Clone)]
struct TranslateTitle {
    pub raw: String,
    pub is_cached: bool,
    pub translated: Option<String>,
}

#[get("/rss")]
async fn rss(req: HttpRequest) -> impl Responder {
    let rss_provider = req
        .app_data::<web::Data<AppState>>()
        .unwrap()
        .rss_provider
        .clone();
    let mut translate_provider = req
        .app_data::<web::Data<AppState>>()
        .unwrap()
        .translate_provider
        .clone();
    let translated_cache_provider = req
        .app_data::<web::Data<AppState>>()
        .unwrap()
        .translated_cache_provider
        .clone();

    // queryパラメータからurlを取得
    let req_query: RssReqQuery = match web::Query::<RssReqQuery>::from_query(req.query_string()) {
        Ok(query) => query.into_inner(),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error (failed to get queries): {}", e));
        }
    };

    // /rss?url= からURLを取得
    let url = req_query.url.clone();

    // クエリパラメータからtoを取得
    let to = match req_query.to.clone() {
        Some(to) => to,
        None => "ja-JP".to_string(),
    };

    // URLからRSSを取得
    let feeds = match rss_provider.get_rss_feeds(url).await {
        Ok(feeds) => feeds,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Error (failed to get rss-feed uri): {}", e));
        }
    };

    // 翻訳用のタイトル集合を用意
    let target_titles: Vec<TranslateTitle> = feeds
        .clone()
        .entries
        .iter()
        .map(|entry| {
            let raw = entry.title.clone().unwrap();
            TranslateTitle {
                raw: raw.content,
                is_cached: false,
                translated: None,
            }
        })
        .collect();
    // キャッシュから翻訳済みのタイトルを取得
    let mut translated_titles: Vec<TranslateTitle> = Vec::new();
    for target_title in target_titles.iter() {
        if translated_cache_provider.clone().is_none() {
            translated_titles.push(TranslateTitle {
                raw: target_title.raw.clone(),
                is_cached: false,
                translated: None,
            });
            continue;
        }

        let raw: String = target_title.raw.clone();
        let translated_cache_provider = translated_cache_provider.clone().unwrap();
        let cached_title = translated_cache_provider.get(raw).await;
        if cached_title.is_err() {
            let err = cached_title.err().unwrap();
            // ログにもエラーを出す
            println!("Error (failed to get title from cache): {}", err);
            return HttpResponse::InternalServerError()
                .body(format!("Error (failed to get title from cache): {}", err));
        }
        let cached_title = cached_title.unwrap();
        match cached_title {
            Some(title) => translated_titles.push(TranslateTitle {
                raw: target_title.raw.clone(),
                is_cached: true,
                translated: Some(title),
            }),
            None => translated_titles.push(TranslateTitle {
                raw: target_title.raw.clone(),
                is_cached: false,
                translated: None,
            }),
        }
    }

    // キャッシュにないタイトルを翻訳
    let translate_target_titles: Vec<String> = translated_titles
        .iter()
        .filter(|title| !title.is_cached)
        .map(|title| title.raw.clone())
        .collect();

    let additional_translated_titles = if translate_target_titles.len() > 0 {
        match translate_provider
            .translate(translate_target_titles, to)
            .await
        {
            Ok(translated) => translated,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Error (failed to translation): {}", e));
            }
        }
    } else {
        Vec::new()
    };

    // 追加で翻訳したタイトルをキャッシュに保存
    if translated_cache_provider.clone().is_some() {
        let translated_cache_provider = translated_cache_provider.clone().unwrap();

        for translated_title in additional_translated_titles.iter() {
            let key = translated_title.raw_text.clone();
            let value = translated_title.translated.clone();
            // 終了を待たないで非同期で行う
            //   エラーが発生した場合はログに残す
            let translated_cache_provider = translated_cache_provider.clone();
            tokio::spawn(async move {
                let set_result = translated_cache_provider.set(key, value).await;
                if set_result.is_err() {
                    let err = set_result.err().unwrap();
                    println!("Error (failed to set title to cache): {}", err);
                }
            });
        }
    }

    // 翻訳済みのタイトルを集合に追加
    let saved_translated_titles: Vec<TranslateTitle> = translated_titles
        .iter()
        .map(|translated_title| {
            let raw = translated_title.raw.clone();
            if translated_title.is_cached {
                return TranslateTitle {
                    raw: raw.clone(),
                    is_cached: true,
                    translated: translated_title.translated.clone(),
                };
            }

            let translated = additional_translated_titles
                .iter()
                .find(|title| title.raw_text == raw)
                .unwrap()
                .translated
                .clone();

            TranslateTitle {
                raw: raw,
                is_cached: true,
                translated: Some(translated),
            }
        })
        .collect();

    // タイトルを翻訳済みに差し替える
    let new_feeds = feeds.clone();
    let new_items = new_feeds
        .entries
        .iter()
        .zip(saved_translated_titles.iter())
        .map(|(item, translated_title)| {
            let mut new_item = item.clone();
            let title = translated_title.translated.clone().unwrap();
            new_item.title = Some(Text {
                content_type: mime::TEXT_PLAIN,
                src: None,
                content: title.trim().to_string(),
            });
            new_item
        })
        .collect();
    let mut feeds = new_feeds.clone();
    feeds.entries = new_items;

    // 元の形式に応じてRSSやAtomに変換してレスポンスとして返す
    let generator: Option<Box<dyn FeedGenerator>> = match feeds.feed_type {
        FeedType::RSS0 => Some(Box::new(RssGenerator::new())),
        FeedType::RSS1 => Some(Box::new(RssGenerator::new())),
        FeedType::RSS2 => Some(Box::new(RssGenerator::new())),
        FeedType::Atom => Some(Box::new(AtomGenerator::new())),
        FeedType::JSON => None,
    };

    if generator.is_none() {
        return HttpResponse::InternalServerError().body("Error: Unsupported feed type");
    }
    let unwraped_generator = generator.unwrap();

    let feed_str = unwraped_generator.generate_feed(feeds);
    let content_type = unwraped_generator.content_type();

    HttpResponse::Ok().content_type(content_type).body(feed_str)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_account_file = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap();
    let project_id = std::env::var("GOOGLE_CLOUD_PROJECT").unwrap();

    let cache_mode = std::env::var("CACHE_MODE");

    let webdav_url = std::env::var("WEB_DAV_URL");
    let webdav_user_id = std::env::var("WEB_DAV_USER_ID");
    let webdav_user_password = std::env::var("WEB_DAV_USER_PASSWORD");

    let s3_endpoint_url = std::env::var("S3_ENDPOINT_URL");
    let s3_bucket_name = std::env::var("S3_BUCKET_NAME");
    let s3_region = std::env::var("S3_REGION");
    let aws_access_key = std::env::var("AWS_ACCESS_KEY_ID");
    let aws_secret_key = std::env::var("AWS_SECRET_ACCESS_KEY");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let rss_provider = rtr::RssProvider::new();
    let translate_provider =
        translate::TranslateProvider::new(translate::TranslateProviderInitConfig {
            project_id: project_id.clone(),
            service_account_json: service_account_file,
        });

    let translated_cache_provider: Option<Box<dyn CacheProvider>> = match cache_mode {
        Ok(cache_mode) => match cache_mode.as_str() {
            "webdav" => Some(Box::new(WebDavCacheProvider::new(
                WebDavCacheProviderOptions {
                    user_id: webdav_user_id.unwrap().clone(),
                    user_password: webdav_user_password.unwrap().clone(),
                    webdav_url: webdav_url.unwrap().clone(),
                },
            ))),
            "s3" => Some(Box::new(S3CacheProvider::new(S3CacheProviderOptions {
                region: s3_region.unwrap(),
                access_key: aws_access_key.unwrap(),
                secret_key: aws_secret_key.unwrap(),
                bucket_name: s3_bucket_name.unwrap(),
                endpoint_url: s3_endpoint_url.unwrap(),
            }))),
            _ => None,
        },
        Err(_) => None,
    };

    let app_state = web::Data::new(AppState {
        rss_provider: rss_provider.clone(),
        translate_provider: translate_provider.clone(),
        translated_cache_provider: translated_cache_provider,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new())
            .app_data(app_state.clone())
            .service(index)
            .service(rss)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
