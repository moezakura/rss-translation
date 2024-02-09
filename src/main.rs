use actix_web::HttpRequest;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use rss_trans::rss as rtr;
use rss_trans::translate;

struct AppState {
    rss_provider: rtr::RssProvider,
    translate_provider: translate::TranslateProvider,
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

    // queryパラメータからurlを取得
    let req_query: RssReqQuery = match web::Query::<RssReqQuery>::from_query(req.query_string()) {
        Ok(query) => query.into_inner(),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error: {}", e));
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
    let channel = match rss_provider.get_rss_feeds(url).await {
        Ok(channel) => channel,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error: {}", e));
        }
    };

    // タイトルを翻訳
    let target_titles = channel
        .items
        .iter()
        .filter_map(|item| item.title.clone())
        .collect();
    let translated_titles = match translate_provider.translate(target_titles, to).await {
        Ok(translated_titles) => translated_titles,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error: {}", e));
        }
    };

    // タイトルを翻訳済みに差し替える
    let new_channel = channel.clone();
    let new_items = new_channel
        .items
        .iter()
        .zip(translated_titles.iter())
        .map(|(item, translated_title)| {
            let mut new_item = item.clone();
            new_item.title = Some(translated_title.clone());
            new_item
        })
        .collect();
    let mut channel = new_channel.clone();
    channel.items = new_items;

    // XMLとして返す
    let new_content = channel.to_string();
    HttpResponse::Ok().body(new_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let service_account_file = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap();
    let project_id = std::env::var("GOOGLE_CLOUD_PROJECT").unwrap();

    let rss_provider = rtr::RssProvider::new();
    let translate_provider =
        translate::TranslateProvider::new(translate::TranslateProviderInitConfig {
            project_id: project_id.clone(),
            service_account_json: service_account_file,
        });

    let app_state = web::Data::new(AppState {
        rss_provider: rss_provider.clone(),
        translate_provider: translate_provider.clone(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(rss)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
