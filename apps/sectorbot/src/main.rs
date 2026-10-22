// serenity's error type makes ECRError large; boxing it is not worth the churn here
#![allow(clippy::result_large_err)]

mod bot;
mod config;
mod ecr;
mod fantasypros;
mod server;
mod store;
mod twitter;
mod yt;

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serenity::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time;

use crate::bot::{AppContext, Handler};
use crate::store::{DataStore, ExternalId, Feed, FeedEntry, FeedEvent};
use crate::twitter::TweetResponse;

const CHANNEL_BUFFER: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    // print cwd
    log::info!(
        "Current working directory: {}",
        std::env::current_dir()?.display()
    );

    let store = store::DataStore::new();

    // List out feeds
    match store.list_feeds() {
        Ok(feeds) => {
            log::info!("Feeds in the data store:");
            for mut feed in feeds {
                log::info!("Feed ID: {}, Name: {}", feed.id, feed.name);
                log::debug!(". Destination: {:?}", feed.destination);
                log::debug!(". Sources: {:?}", feed.sources);
                log::debug!(". Entries: {}", feed.entries.len());

                feed_cleanup(&mut feed, &store);
            }
        }
        Err(e) => {
            log::error!("Failed to list feeds: {}", e);
            return Err(e.into());
        }
    }

    if let Some(path) = &config.feeds_file {
        seed_feeds(&store, path)?;
    }

    log::info!("Data store initialized successfully!");

    let (tx, rx) = mpsc::channel::<FeedEvent>(CHANNEL_BUFFER);

    let yt_client = yt::YoutubeClient::new::<String, String>(
        config.yt_api_key.clone(),
        None, // You can pass a custom HTTP client here if needed
    );

    let app_context = Arc::new(AppContext {
        config,
        store: Arc::new(store),
        new_tweets_sender: tx,
        yt_client: Arc::new(yt_client),
    });

    let server_handle = tokio::spawn({
        let app_context_clone = app_context.clone();
        async move {
            if let Err(e) = server::setup_and_listen(
                app_context_clone.config.http_server_port,
                app_context_clone,
            )
            .await
            {
                log::error!("Failed to start server: {}", e);
            }
        }
    });

    // Producer tasks: poll each configured feed, staggered so they do not all hit the APIs at once
    if app_context.config.enable_feed_fetcher {
        let mut feeds = app_context.store.list_feeds()?;
        feeds.sort_by_key(|feed| feed.id);
        for (i, feed) in feeds.iter().enumerate() {
            let delay = (i > 0).then(|| Duration::from_secs(2 * 60 * i as u64));
            start_feed_fetcher(feed.id, &app_context, delay).await?;
        }
    } else {
        log::info!("Feed fetcher disabled (set ENABLE_FEED_FETCHER=true to poll feeds)");
    }

    // This blocks
    match setup_discord_bot(&app_context, rx).await {
        Ok(_) => log::info!("Discord bot setup successfully!"),
        Err(e) => {
            log::error!("Failed to setup Discord bot: {}", e);
            return Err(e);
        }
    };
    server_handle.await?;

    Ok(())
}

async fn start_feed_fetcher(
    feed_id: u64,
    app_context: &AppContext,
    delay: Option<Duration>,
) -> Result<(), Box<dyn Error>> {
    tokio::spawn({
        let sender_clone = app_context.new_tweets_sender.clone();
        let store_clone = app_context.store.clone();
        let cookie_clone = app_context.config.twitter_cookie.clone();
        let yt_client = Arc::clone(&app_context.yt_client);
        let feed = store_clone.get_feed(feed_id).ok_or("Feed not found")?;
        async move {
            if let Some(delay) = delay {
                log::info!(
                    "Delaying tweet fetcher task. Feed: {}. Delay {:?}",
                    feed.name,
                    delay,
                );
                time::sleep(delay).await;
            }
            log::info!(
                "Starting tweet fetcher task. Feed: {}. Interval {}s",
                feed.name,
                feed.pool_interval
            );
            let mut interval = time::interval(Duration::from_secs(feed.pool_interval));
            loop {
                interval.tick().await;
                match do_feed_fetch(
                    feed_id,
                    &sender_clone,
                    &store_clone,
                    &cookie_clone,
                    &yt_client,
                )
                .await
                {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("Failed to fetch new twitter shit: {err}");
                    }
                };
            }
        }
    });
    Ok(())
}

async fn setup_discord_bot(
    app_context: &AppContext,
    events_receiver: Receiver<FeedEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS // allows to get guild info
        | GatewayIntents::GUILD_MEMBERS // allows to query for members in guilds
        | GatewayIntents::AUTO_MODERATION_EXECUTION
        | GatewayIntents::GUILD_PRESENCES; // allows for updates to the guild's memebr list to be fetched

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&app_context.config.discord_token, intents)
        .event_handler(Handler {
            sleeper_league_id: app_context.config.sleeper_league_id.clone(),
            admin_discord_user_id: app_context.config.admin_discord_user_id,
            yt_client: Arc::clone(&app_context.yt_client),
            events_receiver: Arc::new(Mutex::new(events_receiver)),
            ecr_service: ecr::service::Service::new(&app_context.store).await?,
        })
        .await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    client.start().await?;
    log::info!("Discord bot started successfully!");
    Ok(())
}

/// Adds the feeds from `path` that are not in the data store yet; existing feeds (and their entries) are left alone.
fn seed_feeds(store: &DataStore, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    for feed in config::load_feeds_file(path)?.feeds {
        if store.get_feed(feed.id).is_some() {
            log::debug!(
                "Feed {} ({}) already exists, not seeding",
                feed.id,
                feed.name
            );
        } else {
            log::info!(
                "Seeding feed {} ({}) from {}",
                feed.id,
                feed.name,
                path.display()
            );
            store.save_feed(&feed)?;
        }
    }
    Ok(())
}

async fn do_feed_fetch(
    feed_id: u64,
    tx: &Sender<FeedEvent>,
    store: &DataStore,
    twitter_cookie: &String,
    yt_client: &yt::YoutubeClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut feed = store.get_feed(feed_id).ok_or("Feed not found")?;

    let mut tweets: Vec<TweetResponse> = vec![];
    let mut yt_videos: Vec<yt::Item> = vec![];
    let yt_channel_ids: Vec<String> = feed
        .sources
        .iter()
        .filter_map(|s| {
            if s.platform == Some(store::Platform::Youtube) {
                Some(s.id.to_string())
            } else {
                None
            }
        })
        .collect();
    for source in feed.sources.iter() {
        match source.platform {
            Some(store::Platform::Twitter) => {
                let mut user_tweets =
                    fetch_raw_tweets(source.id.to_string(), twitter_cookie.to_string()).await?;
                log::debug!(
                    "Twitter Timeline for {}. {} entries",
                    source.id,
                    user_tweets.len()
                );
                tweets.append(&mut user_tweets);
            }
            Some(store::Platform::Youtube) => {
                // do after
            }
            _ => {
                log::warn!(
                    "Skipping source {} because it is an unknown platform",
                    source.id
                );
                continue;
            }
        }
    }

    if !yt_channel_ids.is_empty() {
        let channel_uploads_playlist_id = match yt_client
            .list_channels(yt_channel_ids.clone())
            .await
        {
            Ok(list_channels_response) => match list_channels_response {
                yt::Response::ChannelListResponse { items, .. } => {
                    log::debug!("Received metadata on {} channels", items.len());
                    let channels_playlist_ids = items
                        .into_iter()
                        .filter_map(|channel| {
                            channel
                                .uploads_playlist_id()
                                .map(|pid| (channel.title(), pid.clone()))
                        })
                        .collect::<Vec<_>>();
                    if channels_playlist_ids.is_empty() {
                        log::warn!(
                            "No uploads playlists found for the provided channel IDs: {yt_channel_ids:?}"
                        );
                        None
                    } else {
                        Some(channels_playlist_ids)
                    }
                }
                other => {
                    log::warn!(
                        "Failed to list YouTube channels with channels IDs: {yt_channel_ids:?}. Unknown response: {other:?}"
                    );
                    None
                }
            },
            Err(err) => {
                log::error!(
                    "Failed to list YouTube channels with channels IDs: {yt_channel_ids:?}. Error: {err:?}"
                );
                None
            }
        };

        if let Some(channel_uploads_playlist_id) = channel_uploads_playlist_id {
            for (channel_title, playlist_id) in channel_uploads_playlist_id {
                log::debug!("Fetching videos for {channel_title} uploads (playlist {playlist_id})");
                let playlist_videos = match yt_client
                    .list_playlist_items(playlist_id.as_str())
                    .await
                {
                    Ok(r) => r,
                    Err(err) => {
                        log::error!(
                            "Failed to fetch videos for uploads {channel_title} (ID: {playlist_id}): {}",
                            err
                        );
                        continue; // Skip this channel if there's an error
                    }
                };
                if let yt::Response::PlaylistItemListResponse { items, .. } = playlist_videos {
                    log::debug!(
                        "Youtube Uploads for {channel_title} videos in playlist {}",
                        items.len()
                    );
                    for video in items.iter() {
                        log::debug!("Video: {} (ID: {})", video.title(), video.id.id());
                        log::debug!("Published At: {}", video.published_at().unwrap_or_default());
                        log::debug!("URL: https://www.youtube.com/watch?v={}", video.id.id());
                        yt_videos.push(video.clone());
                    }
                } else {
                    log::error!("Unexpected response type from YouTube API");
                }
            }
        }
    };

    tweets.sort_by_key(|tweet| tweet.created_at.to_string());

    let mut new_tweets: Vec<TweetResponse> = vec![];
    let mut new_yt_videos: Vec<yt::Item> = vec![];

    for tweet in tweets {
        if feed
            .entries
            .iter()
            .find(|e| e.id.platform == Some(store::Platform::Twitter) && e.id.id == tweet.id_str)
            .is_none()
        {
            new_tweets.push(tweet);
        }
    }
    for video in yt_videos {
        if feed
            .entries
            .iter()
            .find(|e| {
                e.id.platform == Some(store::Platform::Youtube)
                    && e.id.id == video.video_id().unwrap_or_default()
            })
            .is_none()
        {
            new_yt_videos.push(video);
        }
    }

    if !new_tweets.is_empty() {
        log::info!("Found {} new tweets to add to the feed", new_tweets.len());
        let num_of_new_tweets = new_tweets.len();
        for new_tweet in new_tweets.into_iter() {
            log::info!("----------------------------------");
            log::info!("Created At: {}", new_tweet.created_at);
            log::info!("Text: {}", new_tweet.text());
            log::info!("Link: {}{}", twitter::FX_TWITTER, new_tweet.link());
            log::info!("----------------------------------");
            let entry = store::FeedEntry::from(new_tweet);
            feed.entries.push(entry.clone());
            match tx
                .send(FeedEvent {
                    entry,
                    feed_id,
                    destination: feed.destination.clone(),
                })
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to publish new tweet to channel: {err:?}");
                }
            };
        }

        match store.save_feed(&feed) {
            Ok(_) => {
                log::info!("Feed updated with {num_of_new_tweets} new entries");
            }
            Err(err) => {
                log::error!("Failed to save feed: {}", err);
                return Err(err.into());
            }
        };
    }

    if !new_yt_videos.is_empty() {
        log::info!(
            "Found {} new videos to add to the feed",
            new_yt_videos.len()
        );
        let num_of_new_videos = new_yt_videos.len();
        for new_video in new_yt_videos.into_iter() {
            log::info!("----------------------------------");
            log::info!(
                "Published At: {}",
                new_video.published_at().unwrap_or_default()
            );
            log::info!("Title: {}", new_video.title());
            log::info!(
                "Link: https://www.youtube.com/watch?v={}",
                new_video.id.id()
            );
            log::info!("----------------------------------");
            let entry = store::FeedEntry::from(new_video);
            feed.entries.push(entry.clone());
            match tx
                .send(FeedEvent {
                    entry,
                    feed_id,
                    destination: feed.destination.clone(),
                })
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to publish new video to channel: {err:?}");
                }
            };
        }

        match store.save_feed(&feed) {
            Ok(_) => {
                log::info!("Feed updated with {num_of_new_videos} new entries");
            }
            Err(err) => {
                log::error!("Failed to save feed: {err:?}");
            }
        };
    }

    Ok(())
}

async fn fetch_raw_tweets<T: Display>(
    twitter_username: T,
    twitter_cookie: T,
) -> Result<Vec<TweetResponse>, Box<dyn Error>> {
    let syndication_client = twitter::syndication::Client::new();
    let twitter_timeline = syndication_client
        .fetch_user_timeline(twitter_username.to_string(), twitter_cookie.to_string())
        .await?;
    Ok(
        twitter_timeline //TODO: look into replacing with timeline.tweets()
            .props
            .page_props
            .timeline
            .entries
            .into_iter()
            .filter_map(|e| e.content.tweet)
            .collect(),
    )
}

impl From<TweetResponse> for FeedEntry {
    fn from(tweet: TweetResponse) -> Self {
        let txt = tweet.text();
        FeedEntry {
            id: ExternalId {
                platform: Some(store::Platform::Twitter),
                id: tweet.id_str.clone(),
            },
            title: txt.clone(), // TODO: ???????
            link: format!("{}{}", twitter::FX_TWITTER, tweet.link()),
            source_ts: tweet.created_at.to_utc(),
            content: txt,
            published_at: None, // You can set this to Some(Utc::now()) if you want
            author_icon_url: None, // TODO: ???????
            author_name: None,  // TODO: ???????
            author_id: None,    // TODO: ???????
            embed_image_url: None, // TODO: ???????
        }
    }
}
impl From<yt::Item> for FeedEntry {
    fn from(video: yt::Item) -> Self {
        let video_id = video.video_id().unwrap_or_default();
        let link = video.link().unwrap_or_default();
        FeedEntry {
            id: ExternalId {
                platform: Some(store::Platform::Youtube),
                id: video_id,
            },
            title: video.title(),
            link: link.clone(),
            source_ts: video.published_at().unwrap_or_default(),
            content: video.description(),
            published_at: None, // You can set this to Some(Utc::now()) if you want
            author_icon_url: video.link(),
            author_name: video
                .snippet
                .as_ref()
                .and_then(|snippet| snippet.channel_title.clone()),
            author_id: video
                .snippet
                .as_ref()
                .and_then(|snippet| snippet.channel_id.clone()),
            embed_image_url: video
                .snippet
                .as_ref()
                .map(|s| s.thumbnails.medium.url.clone()),
        }
    }
}

fn feed_cleanup(feed: &mut Feed, store: &DataStore) -> bool {
    // remove duplicate entries
    let mut unique_entries = feed.entries.clone();
    unique_entries.sort_by_key(|e| e.id.clone()); // sort for deduplication
    unique_entries.dedup_by_key(|e| e.id.clone()); // deduplicate by id
    unique_entries.sort_by_key(|e| e.source_ts); // sort by source_ts for save
    log::info!(". Unique Entries: {}", unique_entries.len());
    if unique_entries.len() < feed.entries.len() {
        log::warn!(
            "Feed {} has duplicate entries. Original: {}, Unique: {}",
            feed.name,
            feed.entries.len(),
            unique_entries.len()
        );
        // Update the feed with unique entries
        feed.entries = unique_entries;
        if let Err(e) = store.save_feed(feed) {
            log::error!("Failed to save feed {}: {}", feed.name, e);
        } else {
            log::info!("Feed {} saved with unique entries.", feed.name);
        }
    }
    true // Return true if cleanup was performed
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::{twitter::User, yt::ItemId};
    use tempfile::TempDir;

    #[test]
    fn seeding_adds_new_feeds_and_leaves_existing_ones_alone() {
        let dir = TempDir::new().unwrap();
        let store = DataStore::open(dir.path().join("db")).unwrap();
        let path = std::path::Path::new("feeds.example.json");

        seed_feeds(&store, path).unwrap();
        assert_eq!(2, store.list_feeds().unwrap().len());

        // a feed that has since collected entries must not be reset by the next start-up
        let mut feed = store.get_feed(1).unwrap();
        feed.last_updated = Some("2025-01-15T12:00:00Z".to_string());
        store.save_feed(&feed).unwrap();
        seed_feeds(&store, path).unwrap();
        assert_eq!(2, store.list_feeds().unwrap().len());
        assert_eq!(
            Some("2025-01-15T12:00:00Z".to_string()),
            store.get_feed(1).unwrap().last_updated
        );
    }

    #[test]
    fn seeding_from_a_missing_file_is_an_error_not_a_panic() {
        let dir = TempDir::new().unwrap();
        let store = DataStore::open(dir.path()).unwrap();
        let err = seed_feeds(&store, std::path::Path::new("nope.json")).unwrap_err();
        assert!(err.to_string().contains("nope.json"), "{err}");
    }

    #[test]
    fn test_feed_entry_from_tweet() {
        let tweet = TweetResponse {
            id_str: "1234567890".to_string(),
            full_text: Some("Test tweet".to_string()),
            created_at: chrono::Utc::now().into(),
            permalink: Some("/status/1234567890".to_string()),
            text: "other text".to_string(),
            user: User {
                id_str: "user123".to_string(),
                name: "Test User".to_string(),
                screen_name: "testuser".to_string(),
                profile_image_url_https: "".to_string(),
            },
            lang: "en-us".to_string(),
            favorite_count: None,
            retweet_count: None,
            quoted_tweet: None,
            media_details: None,
        };
        let entry: FeedEntry = tweet.into();
        assert_eq!(entry.id.id, "1234567890");
        assert_eq!(entry.title, "Test tweet");
        assert_eq!(entry.link, "https://fxtwitter.com/status/1234567890");
    }

    #[test]
    fn test_feed_entry_from_yt_video() {
        let playlist_item_json = r#"
        {
            "kind": "youtube#playlistItem",
            "etag": "l-yXl5XjmiLRHuP2MN6E1K8ewbM",
            "id": "VVVKNUp4ZFhRNnhoNDhHTDhrR1FKVDB3LkxXZUs4akpwSzNz",
            "snippet": {
                "publishedAt": "2025-07-27T21:18:21Z",
                "channelId": "UCexampleRoverBanter0001",
                "title": "Rovers Camp: rookie delivers + big play in practice",
                "description": "Sam and Alex break down the last two Rovers practices and a deep ball. That and a whole lot more!\n\n#Rovers #football #camp",
                "thumbnails": {
                "default": {
                    "url": "https://i.ytimg.com/vi/exampleVid01/default.jpg",
                    "width": 120,
                    "height": 90
                },
                "medium": {
                    "url": "https://i.ytimg.com/vi/exampleVid01/mqdefault.jpg",
                    "width": 320,
                    "height": 180
                },
                "high": {
                    "url": "https://i.ytimg.com/vi/exampleVid01/hqdefault.jpg",
                    "width": 480,
                    "height": 360
                },
                "standard": {
                    "url": "https://i.ytimg.com/vi/exampleVid01/sddefault.jpg",
                    "width": 640,
                    "height": 480
                },
                "maxres": {
                    "url": "https://i.ytimg.com/vi/exampleVid01/maxresdefault.jpg",
                    "width": 1280,
                    "height": 720
                }
                },
                "channelTitle": "Rover Banter",
                "playlistId": "UUexampleRoverBanter0001",
                "position": 0,
                "resourceId": {
                    "kind": "youtube#video",
                    "videoId": "exampleVid01"
                },
                "videoOwnerChannelTitle": "Rover Banter",
                "videoOwnerChannelId": "UCexampleRoverBanter0001"
            },
            "contentDetails": {
                "videoId": "exampleVid01",
                "videoPublishedAt": "2025-07-27T21:18:21Z"
            }
        }
        "#;

        let video: yt::Item = serde_json::from_str(playlist_item_json).unwrap();
        // assert_eq!(video.clone().snippet.unwrap().resource_id.unwrap(), ItemId::Obj {
        //     kind: yt::ItemKind::Video,
        //     video_id: Some("exampleVid01".to_string()),
        //     channel_id: None,
        // }); // FIXME: this fails for some reason, the deserialization isn't working
        let entry: FeedEntry = video.into();
        assert_eq!(entry.id.id, "exampleVid01");
        assert_eq!(
            entry.title,
            "Rovers Camp: rookie delivers + big play in practice"
        );
        assert_eq!(entry.link, "https://www.youtube.com/watch?v=exampleVid01");
    }

    // #[tokio::test]
    // async fn test_fetch_raw_feed() {
    //     config::load_config().expect("Failed to load config");

    //     let feeds: Vec<Feed> = reqwest::get("http://35.211.148.134/feeds").await.unwrap().json().await.unwrap();

    //     assert!(!feeds.is_empty(), "No feeds found");
    //     for feed in feeds {
    //         log::info!("Feed ID: {}, Name: {}", feed.id, feed.name);
    //         log::info!(". Destination: {:?}", feed.destination);
    //         log::info!(". Sources: {:?}", feed.sources);
    //         log::info!(". Entries:         {}", feed.entries.len());
    //         log::info!(". Twitter Entries: {}", feed.entries.iter().filter(|e| e.id.platform == Some(store::Platform::Twitter)).count());
    //         log::info!(". YouTube Entries: {}", feed.entries.iter().filter(|e| e.id.platform == Some(store::Platform::Youtube)).count());
    //         log::info!(". Other Entries:   {}", feed.entries.iter().filter(|e| e.id.platform != Some(store::Platform::Twitter) && e.id.platform != Some(store::Platform::Youtube)).count());
    //     }

    //     panic!("Test completed, check the logs for output");
    // }
}
