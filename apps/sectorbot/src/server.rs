use std::{error::Error, net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json, Router,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
};
use reqwest::StatusCode;
use rss::Channel;

use crate::{
    bot::AppContext,
    store::{Feed, FeedEntry},
};

pub async fn setup_and_listen(
    port: u16,
    app_context: Arc<AppContext>,
) -> Result<(), Box<dyn Error>> {
    // Create the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/feeds", get(fetch_feeds))
        .route("/feeds/{feed_id}", get(fetch_rss_feed))
        .layer(Extension(app_context));
    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server running at http://{}", addr);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check(
    Extension(state): Extension<Arc<AppContext>>,
) -> Result<Json<Health>, AppError> {
    Ok(Json(Health {
        status: "ok".to_string(),
        message: "Server is running".to_string(),
        yt_credits_used: state.yt_client.credits_consumed(),
    }))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Health {
    pub status: String,
    pub message: String,
    pub yt_credits_used: usize,
}

async fn fetch_feeds(
    // Path(date): Path<NaiveDate>,
    // Query(slate_filters): Query<SlateFilters>,
    Extension(state): Extension<Arc<AppContext>>,
) -> Result<Json<Vec<Feed>>, AppError> {
    match state.store.list_feeds() {
        Ok(feeds) => Ok(Json(feeds)),
        Err(err) => Err(AppError::CustomError(format!(
            "failed to fetch_slates: {err}",
        ))),
    }
}

async fn fetch_rss_feed(
    Path(feed_name): Path<String>,
    // Query(slate_filters): Query<SlateFilters>,
    Extension(state): Extension<Arc<AppContext>>,
) -> impl IntoResponse {
    log::info!("Fetching RSS feed with feed_name={feed_name}");
    let feed_id = match feed_name.as_str() {
        "giants" => 1,
        "nfl" => 2,
        other => {
            log::warn!("Unknown feed name: {}", other);
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain")
                .body("feed not found".to_string())
                .unwrap();
        }
    };

    let response = match state.store.get_feed(feed_id) {
        Some(feed) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/rss+xml")
            .body(Into::<rss::Channel>::into(feed).to_string())
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body("feed not found".to_string())
            .unwrap(),
    };
    response
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum AppError {
    BadRequest(String),
    NotFound(String),
    CustomError(String),
}

// Implement `IntoResponse` for `AppError` to customize the HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            AppError::CustomError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
        }
    }
}

impl From<Feed> for rss::Channel {
    fn from(feed: Feed) -> Self {
        let feed_link = format!("http://host-de504cfe.example.invalid/feeds/{}", feed.name);
        let mut channel = Channel::default();
        channel.set_title(feed.name.clone());
        channel.set_description(feed.description.clone());
        channel.set_link(feed_link);
        channel.set_language("en-US".to_string());
        channel.set_last_build_date(chrono::Utc::now().to_rfc2822()); // consider handlign differently
        channel.set_generator("Sectorbot RSS Generator".to_string());
        channel.set_ttl(1.to_string()); // Time to live in minutes
        // channel.set_image(rss::Image {
        //     url: "http://".to_string(),
        //     title: "Sectorbot Feed".to_string(),
        //     link: "http://".to_string(),
        //     width: None,
        //     height: None,
        //     description: None,
        //     ..Default::default()
        // });
        // channel.set_web_master("".to_string());
        channel.set_copyright("Copyright © 2025 Sectorbot".to_string());
        channel.set_pub_date(chrono::Utc::now().to_rfc2822()); // consider handlign differently
        // channel.set_docs("https://sectorbot.com/rss".to_string());
        // channel.set_managing_editor("".to_string());

        // let extensions = ExtensionMap::default();
        // channel.set_extensions(extensions);

        let mut entries = feed.entries;
        entries.sort_by_key(|e| e.source_ts);
        entries.reverse(); // Most recent first

        for entry in entries {
            channel.items.push(entry.into());
        }
        channel
    }
}

impl From<FeedEntry> for rss::Item {
    fn from(entry: FeedEntry) -> Self {
        rss::Item {
            title: Some(html_escape::decode_html_entities(&entry.title).into_owned()),
            link: Some(entry.link.clone()),
            description: Some(html_escape::decode_html_entities(&entry.content).into_owned()),
            // author: todo!(),
            // categories: todo!(),
            // comments: todo!(),
            // enclosure: todo!(),
            guid: Some(rss::Guid {
                // should I actually make the guid be just the twitter link?
                value: entry.id.to_string(),
                permalink: false,
            }),
            pub_date: Some(
                entry.source_ts.to_rfc2822(), // Convert to RFC 2822 format
            ),
            // source: todo!(),
            // content: todo!(),
            // extensions: todo!(),
            // itunes_ext: todo!(),
            // dublin_core_ext: todo!(),
            ..Default::default()
        }
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use crate::store::{ExternalId, Feed, FeedEntry, Platform};
    use chrono::{DateTime, Duration, Utc};

    #[test]
    fn test_rss_feed_conversion() {
        let entry = FeedEntry {
            id: ExternalId {
                platform: Some(Platform::Twitter),
                id: "1234567890".to_string(),
            },
            title: "Test Entry".to_string(),
            link: "http://example.com/test-entry".to_string(),
            content: "- It’s Malik Nabers’ world &amp; we’re just living in it".to_string(),
            source_ts: DateTime::parse_from_rfc3339("2023-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            published_at: None,
        };

        let feed = Feed {
            id: 1,
            name: "Test Feed".to_string(),
            description: "A feed for testing".to_string(),
            entries: vec![entry],
            sources: vec![],
            last_updated: None,
            destination: ExternalId::new(Platform::Discord, "176082542134324141".to_string()),
            pool_interval: 60,
            quiet_time: "".to_string(),
        };

        let rss_channel: rss::Channel = feed.into();
        assert_eq!(rss_channel.title, "Test Feed");
        assert!(!rss_channel.items.is_empty());
        assert_eq!(1, rss_channel.items.len());
        assert_eq!(
            "- It’s Malik Nabers’ world & we’re just living in it",
            &rss_channel.items[0].description.clone().unwrap()
        );
    }
}
