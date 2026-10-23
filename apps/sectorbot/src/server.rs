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
    store::{Feed, FeedEntry, League},
};

/// The HTTP API: health, the configured feeds (JSON and RSS) and the known leagues.
pub fn router(app_context: Arc<AppContext>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/feeds", get(fetch_feeds))
        .route("/feeds/{feed_id}", get(fetch_rss_feed))
        .route("/leagues", get(fetch_leagues))
        .layer(Extension(app_context))
}

pub async fn setup_and_listen(
    addr: SocketAddr,
    app_context: Arc<AppContext>,
) -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("HTTP Server listening on http://{}", listener.local_addr()?);
    axum::serve(listener, router(app_context)).await?;
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
            "failed to fetch_feeds: {err}",
        ))),
    }
}

async fn fetch_rss_feed(
    Path(feed_name): Path<String>,
    // Query(slate_filters): Query<SlateFilters>,
    Extension(state): Extension<Arc<AppContext>>,
) -> impl IntoResponse {
    log::info!("Fetching RSS feed with feed_name={feed_name}");
    let feed = state
        .store
        .list_feeds()
        .unwrap_or_default()
        .into_iter()
        .find(|feed| feed_slug(&feed.name) == feed_name || feed.id.to_string() == feed_name);

    match feed {
        Some(feed) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/rss+xml")
            .body(feed_to_channel(feed, &state.config.public_base_url).to_string())
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body("feed not found".to_string())
            .unwrap(),
    }
}

async fn fetch_leagues(
    Extension(state): Extension<Arc<AppContext>>,
) -> Result<Json<Vec<League>>, AppError> {
    match state.store.list_leagues() {
        Ok(feeds) => Ok(Json(feeds)),
        Err(err) => Err(AppError::CustomError(format!(
            "failed to fetch_leagues: {err}",
        ))),
    }
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

/// URL-friendly feed name: "Rovers News" -> "rovers-news"
pub(crate) fn feed_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub(crate) fn feed_to_channel(feed: Feed, public_base_url: &str) -> Channel {
    let feed_link = format!(
        "{}/feeds/{}",
        public_base_url.trim_end_matches('/'),
        feed_slug(&feed.name)
    );
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

#[cfg(test)]
#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;
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
            content: "- It’s Casey Catcher’s world &amp; we’re just living in it".to_string(),
            source_ts: DateTime::parse_from_rfc3339("2023-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            published_at: None,
            author_icon_url: None,
            author_name: None,
            author_id: None,
            embed_image_url: None,
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

        let rss_channel = crate::server::feed_to_channel(feed, "http://localhost:8080/");
        assert_eq!(rss_channel.title, "Test Feed");
        assert!(!rss_channel.items.is_empty());
        assert_eq!(1, rss_channel.items.len());
        assert_eq!("http://localhost:8080/feeds/test-feed", rss_channel.link);
        assert_eq!(
            "- It’s Casey Catcher’s world & we’re just living in it",
            &rss_channel.items[0].description.clone().unwrap()
        );
    }

    #[test]
    fn test_feed_slug() {
        assert_eq!("rovers-news", crate::server::feed_slug("Rovers News"));
        assert_eq!("nfl-news", crate::server::feed_slug("  NFL  News! "));
        assert_eq!("feed-2", crate::server::feed_slug("Feed #2"));
    }

    // --- the HTTP API, served on an ephemeral port --------------------------------------------

    use crate::{bot::AppContext, config::Config, store::DataStore, yt::YoutubeClient};
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tempfile::TempDir;

    fn config() -> Config {
        Config {
            http_server_port: 0,
            http_bind_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            public_base_url: "https://bot.example.invalid/".to_string(),
            discord_token: "token".into(),
            twitter_cookie: "cookie".into(),
            yt_api_key: "key".into(),
            sleeper_league_id: None,
            admin_discord_user_id: None,
            feeds_file: None,
            enable_feed_fetcher: false,
        }
    }

    fn entry(id: &str, title: &str, minute: i64) -> FeedEntry {
        FeedEntry {
            id: ExternalId::new(Platform::Twitter, id.to_string()),
            title: title.to_string(),
            link: format!("https://example.invalid/{id}"),
            content: format!("{title} & more"),
            source_ts: DateTime::from_timestamp(1_736_900_000 + minute * 60, 0).unwrap(),
            published_at: None,
            author_icon_url: None,
            author_name: Some("Sam Sample".into()),
            author_id: None,
            embed_image_url: None,
        }
    }

    /// serves the API for a store with two feeds and one league; returns its base url
    async fn serve() -> (String, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = DataStore::open(dir.path()).unwrap();
        for (id, name) in [(1u64, "Rovers News"), (2, "League News")] {
            store
                .save_feed(&Feed {
                    id,
                    name: name.to_string(),
                    description: format!("{name} description"),
                    sources: vec![],
                    last_updated: None,
                    destination: ExternalId::new(Platform::Discord, "555".into()),
                    pool_interval: 300,
                    quiet_time: String::new(),
                    entries: if id == 1 {
                        vec![entry("1", "older", 1), entry("2", "newer", 2)]
                    } else {
                        vec![]
                    },
                })
                .unwrap();
        }
        store
            .save_league(&League {
                guild_id: "guild-1".into(),
                current_season: 2025,
                league_id_map: HashMap::new(),
                member_map: HashMap::new(),
            })
            .unwrap();
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        let context = Arc::new(AppContext {
            config: config(),
            store: Arc::new(store),
            new_tweets_sender: tx,
            yt_client: Arc::new(YoutubeClient::new::<String, String>("key".into(), None)),
        });
        let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, router(context)).await });
        (format!("http://{addr}"), dir)
    }

    #[tokio::test]
    async fn health_reports_ok_and_the_youtube_credits_used() {
        let (base, _dir) = serve().await;
        let health: Health = reqwest::get(format!("{base}/health"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!("ok", health.status);
        assert_eq!(0, health.yt_credits_used);
    }

    #[tokio::test]
    async fn feeds_are_listed_as_json() {
        let (base, _dir) = serve().await;
        let feeds: Vec<Feed> = reqwest::get(format!("{base}/feeds"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let mut names: Vec<String> = feeds.into_iter().map(|f| f.name).collect();
        names.sort();
        assert_eq!(vec!["League News", "Rovers News"], names);
    }

    #[tokio::test]
    async fn a_feed_is_served_as_rss_by_slug_or_by_id() {
        let (base, _dir) = serve().await;
        for key in ["rovers-news", "1"] {
            let response = reqwest::get(format!("{base}/feeds/{key}")).await.unwrap();
            assert_eq!(200, response.status().as_u16(), "{key}");
            assert_eq!("application/rss+xml", response.headers()["content-type"]);
            let channel =
                rss::Channel::read_from(response.bytes().await.unwrap().as_ref()).unwrap();
            assert_eq!("Rovers News", channel.title);
            assert_eq!(
                "https://bot.example.invalid/feeds/rovers-news",
                channel.link
            );
            assert_eq!(2, channel.items.len());
            assert_eq!(
                Some("newer"),
                channel.items[0].title.as_deref(),
                "newest first"
            );
        }
    }

    #[tokio::test]
    async fn an_unknown_feed_or_route_is_a_404() {
        let (base, _dir) = serve().await;
        let missing = reqwest::get(format!("{base}/feeds/nope")).await.unwrap();
        assert_eq!(404, missing.status().as_u16());
        assert_eq!("feed not found", missing.text().await.unwrap());
        assert_eq!(
            404,
            reqwest::get(format!("{base}/nothing-here"))
                .await
                .unwrap()
                .status()
                .as_u16()
        );
    }

    #[tokio::test]
    async fn leagues_are_listed() {
        let (base, _dir) = serve().await;
        let leagues: Vec<League> = reqwest::get(format!("{base}/leagues"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(1, leagues.len());
        assert_eq!("guild-1", leagues[0].guild_id);
    }
}
