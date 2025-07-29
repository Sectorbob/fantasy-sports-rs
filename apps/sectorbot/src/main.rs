mod bot;
mod config;
mod custom_feed_generate;
mod server;
mod store;
mod tweeter;
mod yt;

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serenity::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time;

use crate::bot::{AppContext, Handler};
use crate::store::{DataStore, ExternalId, FeedEvent};
use crate::tweeter::TweetResponse;
use crate::yt::YoutubeClient;

const SERVER_PORT: u16 = 8080;
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
            for feed in feeds {
                log::info!("Feed ID: {}, Name: {}", feed.id, feed.name);
                log::info!(". Destination: {:?}", feed.destination);
                log::info!(". Sources: {:?}", feed.sources);
                log::info!(". Entries: {}", feed.entries.len());
            }
        }
        Err(e) => {
            log::error!("Failed to list feeds: {}", e);
            return Err(e);
        }
    }

    log::info!("Data store initialized successfully!");

    let (tx, rx) = mpsc::channel::<FeedEvent>(CHANNEL_BUFFER);

    let yt_client = yt::YoutubeClient::new::<String, String>(
        config.yt_api_key.clone(),
        None, // You can pass a custom HTTP client here if needed
    );

    let app_context = Arc::new(AppContext {
        config,
        store,
        new_tweets_sender: tx,
        yt_client: Arc::new(yt_client),
    });

    let server_handle = tokio::spawn({
        let app_context_clone = app_context.clone();
        async move {
            if let Err(e) = server::setup_and_listen(SERVER_PORT, app_context_clone).await {
                log::error!("Failed to start server: {}", e);
            }
        }
    });

    // Producer task: generates messages
    start_feed_fetcher(1, &app_context, None).await?;
    start_feed_fetcher(2, &app_context, Some(Duration::from_secs(2 * 60))).await?;

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
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&app_context.config.discord_token, intents)
        .event_handler(Handler {
            events_receiver: Arc::new(Mutex::new(events_receiver)),
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

const FX_TWITTER: &str = "https://fxtwitter.com";

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
                log::info!(
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
        let playlist_ids = match yt_client.list_channels(yt_channel_ids).await? {
            yt::Response::ChannelListResponse {
                etag: _,
                next_page_token: _,
                page_info: _,
                items,
            } => {
                // log::info!("Found {} channels", items.len());
                items
                    .into_iter()
                    .filter_map(|channel| channel.uploads_playlist_id())
                    .collect::<Vec<_>>()
            }
            _ => {
                vec![]
            }
        };

        for playlist_id in playlist_ids {
            log::debug!("Fetching videos for playlist ID: {}", playlist_id);
            let playlist_videos = yt_client.list_playlist_items(playlist_id.as_str()).await?;
            if let yt::Response::PlaylistItemListResponse {
                etag: _,
                next_page_token: _,
                page_info: _,
                items,
            } = playlist_videos
            {
                log::info!("Found {} videos in playlist {}", items.len(), playlist_id);
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
    };

    tweets.sort_by_key(|tweet| tweet.created_at.to_string());

    let mut new_tweets: Vec<TweetResponse> = vec![];
    let mut new_yt_videos: Vec<yt::Item> = vec![];

    for tweet in tweets {
        if let None = feed
            .entries
            .iter()
            .find(|e| e.id.platform == Some(store::Platform::Twitter) && e.id.id == tweet.id_str)
        {
            new_tweets.push(tweet);
        }
    }
    for video in yt_videos {
        if let None = feed
            .entries
            .iter()
            .find(|e| e.id.platform == Some(store::Platform::Youtube) && e.id.id == video.id.id())
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
            log::info!("Text: {}", new_tweet.full_text);
            log::info!("Link: {FX_TWITTER}{}", new_tweet.permalink);
            log::info!("----------------------------------");
            let entry = store::FeedEntry {
                id: ExternalId {
                    platform: Some(store::Platform::Twitter),
                    id: new_tweet.id_str.clone(),
                },
                title: new_tweet.full_text.clone(),
                link: format!("{FX_TWITTER}{}", new_tweet.permalink),
                source_ts: new_tweet.created_at.to_utc(),
                content: new_tweet.full_text.clone(),
                published_at: None, // You can set this to Some(Utc::now()) if you want
            };
            feed.entries.push(entry.clone());
            tx.send(FeedEvent {
                entry,
                feed_id,
                destination: feed.destination.clone(),
            })
            .await?;
        }

        store.save_feed(&feed)?;
        log::info!("Feed updated with {num_of_new_tweets} new entries");
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
            let entry = store::FeedEntry {
                id: ExternalId {
                    id: new_video.id.id(),
                    platform: Some(store::Platform::Youtube),
                },
                title: new_video.title(),
                link: format!("https://www.youtube.com/watch?v={}", new_video.id.id()), //new_video.link().unwrap_or_default(),
                source_ts: new_video.published_at().unwrap_or_default(),
                content: new_video.description(),
                published_at: None, // You can set this to Some(Utc::now()) if you want
            };
            feed.entries.push(entry.clone());
            tx.send(FeedEvent {
                entry,
                feed_id,
                destination: feed.destination.clone(),
            })
            .await?;
        }

        store.save_feed(&feed)?;
        log::info!("Feed updated with {num_of_new_videos} new entries");
    }

    Ok(())
}

async fn fetch_raw_tweets<T: Display>(
    twitter_username: T,
    twitter_cookie: T,
) -> Result<Vec<TweetResponse>, Box<dyn Error>> {
    let twitter_timeline =
        tweeter::fetch_user_timeline(twitter_username.to_string(), twitter_cookie.to_string())
            .await?;
    Ok(twitter_timeline
        .props
        .page_props
        .timeline
        .entries
        .into_iter()
        .filter_map(|e| e.content.tweet)
        .collect())
}
