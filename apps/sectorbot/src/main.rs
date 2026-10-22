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
            let addr = std::net::SocketAddr::new(
                app_context_clone.config.http_bind_address,
                app_context_clone.config.http_server_port,
            );
            if let Err(e) = server::setup_and_listen(addr, app_context_clone).await {
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
        let twitter_client = twitter::syndication::Client::new();
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
                if feed.is_quiet_at(chrono::Utc::now()) {
                    log::debug!(
                        "Feed {} is in its quiet hours ({})",
                        feed.name,
                        feed.quiet_time
                    );
                    continue;
                }
                match do_feed_fetch(
                    feed_id,
                    &sender_clone,
                    &store_clone,
                    &cookie_clone,
                    &twitter_client,
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
    twitter_client: &twitter::syndication::Client,
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
                // one unreachable account must not stop the other sources of the feed
                let mut user_tweets = match fetch_raw_tweets(
                    twitter_client,
                    source.id.to_string(),
                    twitter_cookie.to_string(),
                )
                .await
                {
                    Ok(tweets) => tweets,
                    Err(err) => {
                        log::error!("Failed to fetch the timeline of {}: {err}", source.id);
                        continue;
                    }
                };
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

    // anything the feed already has, or that appears twice in this run, is not new
    let mut seen: std::collections::HashSet<(Option<store::Platform>, String)> = feed
        .entries
        .iter()
        .map(|e| (e.id.platform.clone(), e.id.id.clone()))
        .collect();
    for tweet in tweets {
        if seen.insert((Some(store::Platform::Twitter), tweet.id_str.clone())) {
            new_tweets.push(tweet);
        }
    }
    for video in yt_videos {
        if seen.insert((
            Some(store::Platform::Youtube),
            video.video_id().unwrap_or_default(),
        )) {
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
    syndication_client: &twitter::syndication::Client,
    twitter_username: T,
    twitter_cookie: T,
) -> Result<Vec<TweetResponse>, Box<dyn Error>> {
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
            author_icon_url: None, // the channel's icon is not part of a playlist item
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
    use tokio::sync::mpsc;
    use wiremock::{
        Mock, MockServer, Request, Respond, ResponseTemplate,
        matchers::{method, path, path_regex},
    };

    fn testdata(rel: &str) -> String {
        std::fs::read_to_string(format!("{}/testdata/{rel}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    /// The YouTube channels endpoint: only the channels that were asked for (`id=a,b`) come back.
    struct Channels;
    impl Respond for Channels {
        fn respond(&self, req: &Request) -> ResponseTemplate {
            let wanted: Vec<String> = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.split(',').map(String::from).collect())
                .unwrap_or_default();
            let mut all: serde_json::Value =
                serde_json::from_str(&testdata("yt/list_channels.json")).unwrap();
            all["items"]
                .as_array_mut()
                .unwrap()
                .retain(|c| wanted.contains(&c["id"].as_str().unwrap().to_string()));
            ResponseTemplate::new(200).set_body_json(all)
        }
    }

    struct Fixtures {
        twitter: MockServer,
        youtube: MockServer,
    }
    async fn fixtures() -> Fixtures {
        let twitter = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/srv/timeline-profile/screen-name/.*"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(testdata("timeline.html"), "text/html"),
            )
            .mount(&twitter)
            .await;
        let youtube = MockServer::start().await;
        Mock::given(path("/channels"))
            .respond_with(Channels)
            .mount(&youtube)
            .await;
        Mock::given(path("/playlistItems"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(testdata("yt/list_playlist_items.json"), "application/json"),
            )
            .mount(&youtube)
            .await;
        Fixtures { twitter, youtube }
    }

    fn feed(id: u64, sources: &[&str]) -> Feed {
        Feed {
            id,
            name: format!("Feed {id}"),
            description: String::new(),
            sources: sources
                .iter()
                .map(|s| ExternalId::try_from(*s).unwrap())
                .collect(),
            last_updated: None,
            destination: ExternalId::try_from("discord:555").unwrap(),
            pool_interval: 60,
            quiet_time: String::new(),
            entries: vec![],
        }
    }

    /// run one fetch of `feed` and return the events it published
    async fn fetch(
        fixtures: &Fixtures,
        store: &DataStore,
        feed_id: u64,
    ) -> (Result<(), Box<dyn std::error::Error>>, Vec<FeedEvent>) {
        let (tx, mut rx) = mpsc::channel::<FeedEvent>(100);
        let twitter = twitter::syndication::Client::new().with_base_url(fixtures.twitter.uri());
        let youtube = yt::YoutubeClient::new("key", Some(fixtures.youtube.uri()));
        let result = do_feed_fetch(
            feed_id,
            &tx,
            store,
            &"cookie".to_string(),
            &twitter,
            &youtube,
        )
        .await;
        drop(tx);
        let mut events = vec![];
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        (result, events)
    }

    fn store() -> (DataStore, TempDir) {
        let dir = TempDir::new().unwrap();
        (DataStore::open(dir.path()).unwrap(), dir)
    }

    #[tokio::test]
    async fn new_tweets_are_published_once_in_order_and_remembered() {
        let fixtures = fixtures().await;
        let (store, _dir) = store();
        store
            .save_feed(&feed(1, &["twitter:SampleScribe"]))
            .unwrap();

        let (result, events) = fetch(&fixtures, &store, 1).await;

        result.unwrap();
        assert_eq!(5, events.len());
        assert!(
            events
                .iter()
                .all(|e| e.feed_id == 1 && e.destination.id == "555")
        );
        let times: Vec<_> = events.iter().map(|e| e.entry.source_ts).collect();
        assert!(times.is_sorted(), "oldest first: {times:?}");
        assert!(events[0].entry.title.starts_with("Sample update #"));
        assert_eq!(5, store.get_feed(1).unwrap().entries.len());

        // the same timeline again has nothing new
        let (result, events) = fetch(&fixtures, &store, 1).await;
        result.unwrap();
        assert!(events.is_empty(), "{} repeated", events.len());
        assert_eq!(5, store.get_feed(1).unwrap().entries.len());
    }

    #[tokio::test]
    async fn new_videos_are_published_once_without_a_bogus_author_icon() {
        let fixtures = fixtures().await;
        let (store, _dir) = store();
        store
            .save_feed(&feed(2, &["youtube:UCexampleRoverBanter0001"]))
            .unwrap();

        let (result, events) = fetch(&fixtures, &store, 2).await;

        result.unwrap();
        assert_eq!(5, events.len(), "five videos in the uploads playlist");
        assert!(
            events[0]
                .entry
                .link
                .starts_with("https://www.youtube.com/watch?v=")
        );
        assert_eq!(
            Some("Rover Banter".to_string()),
            events[0].entry.author_name
        );
        // this used to be the video's own link, which Discord rejects as an icon
        assert!(events.iter().all(|e| e.entry.author_icon_url.is_none()));
        assert_eq!(5, store.get_feed(2).unwrap().entries.len());

        let (_, again) = fetch(&fixtures, &store, 2).await;
        assert!(again.is_empty());
    }

    #[tokio::test]
    async fn a_failing_twitter_account_does_not_stop_the_other_sources() {
        let fixtures = fixtures().await;
        fixtures.twitter.reset().await; // every path is now a 404
        let (store, _dir) = store();
        store
            .save_feed(&feed(
                3,
                &[
                    "twitter:Gone",
                    "youtube:UCexampleRoverBanter0001",
                    "discord:1",
                ],
            ))
            .unwrap();

        let (result, events) = fetch(&fixtures, &store, 3).await;

        result.expect("the feed run succeeds");
        assert_eq!(5, events.len(), "the youtube videos still arrived");
        assert!(
            events
                .iter()
                .all(|e| e.entry.id.platform == Some(store::Platform::Youtube))
        );
    }

    #[tokio::test]
    async fn the_same_video_in_two_channels_is_published_once() {
        let fixtures = fixtures().await;
        let (store, _dir) = store();
        // both channels' uploads playlists answer with the same five videos in this fixture
        store
            .save_feed(&feed(
                4,
                &[
                    "youtube:UCexampleRoverBanter0001",
                    "youtube:UCexampleRoverInsider001",
                ],
            ))
            .unwrap();

        let (_, events) = fetch(&fixtures, &store, 4).await;

        assert_eq!(5, events.len());
    }

    #[tokio::test]
    async fn fetching_an_unknown_feed_is_an_error() {
        let fixtures = fixtures().await;
        let (store, _dir) = store();
        let (result, events) = fetch(&fixtures, &store, 99).await;
        assert!(result.unwrap_err().to_string().contains("Feed not found"));
        assert!(events.is_empty());
    }

    #[test]
    fn cleaning_a_feed_removes_duplicate_entries_and_saves_it() {
        let (store, _dir) = store();
        let mut feed = feed(5, &[]);
        let entry = |id: &str, minute: i64| FeedEntry {
            id: ExternalId::try_from(format!("twitter:{id}")).unwrap(),
            title: id.into(),
            link: String::new(),
            source_ts: chrono::DateTime::from_timestamp(1_700_000_000 + minute * 60, 0).unwrap(),
            content: String::new(),
            published_at: None,
            author_icon_url: None,
            author_name: None,
            author_id: None,
            embed_image_url: None,
        };
        feed.entries = vec![entry("b", 2), entry("a", 1), entry("b", 2), entry("c", 3)];
        store.save_feed(&feed).unwrap();

        assert!(feed_cleanup(&mut feed, &store));

        let ids: Vec<&str> = feed.entries.iter().map(|e| e.id.id.as_str()).collect();
        assert_eq!(vec!["a", "b", "c"], ids, "unique, oldest first");
        assert_eq!(3, store.get_feed(5).unwrap().entries.len());
    }

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
