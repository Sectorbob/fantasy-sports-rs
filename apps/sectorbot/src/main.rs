mod bot;
mod config;
mod custom_feed_generate;
mod store;
mod tweeter;

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use serenity::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time;

use crate::bot::{AppContext, Handler};
use crate::store::{DataStore, FeedEvent};
use crate::tweeter::TweetResponse;

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

    let store = store::DataStore::new();

    // giants(&store).await?;
    // nfl(&store).await?;

    if let Some(feed) = store.get_feed(1) {
        log::info!("Feed found entries: {}", feed.entries.len());
    }

    log::info!("Data store initialized successfully!");

    let (tx, rx) = mpsc::channel::<FeedEvent>(CHANNEL_BUFFER);

    let app_context = Arc::new(AppContext {
        config,
        store,
        new_tweets_sender: tx,
    });

    // Producer task: generates messages
    start_tweet_fetcher(1, &app_context, None).await?;
    start_tweet_fetcher(2, &app_context, Some(Duration::from_secs(2 * 60))).await?;

    // This blocks
    match setup_discord_bot(&app_context, rx).await {
        Ok(_) => log::info!("Discord bot setup successfully!"),
        Err(e) => {
            log::error!("Failed to setup Discord bot: {}", e);
            return Err(e);
        }
    };

    Ok(())
}

async fn start_tweet_fetcher(
    feed_id: u64,
    app_context: &AppContext,
    delay: Option<Duration>,
) -> Result<(), Box<dyn Error>> {
    tokio::spawn({
        let sender_clone = app_context.new_tweets_sender.clone();
        let store_clone = app_context.store.clone();
        let cookie_clone = app_context.config.twitter_cookie.clone();
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
                match do_twitter_fetch(feed_id, &sender_clone, &store_clone, &cookie_clone).await {
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

async fn do_twitter_fetch(
    feed_id: u64,
    tx: &Sender<FeedEvent>,
    store: &DataStore,
    twitter_cookie: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut feed = store.get_feed(feed_id).ok_or("Feed not found")?;

    let mut tweets: Vec<TweetResponse> = vec![];
    for twitter_username in feed.sources.iter() {
        let mut user_tweets = fetch_raw_tweets(twitter_username, twitter_cookie).await?;
        log::info!(
            "Twitter Timeline for {}. {} entries",
            twitter_username,
            user_tweets.len()
        );
        tweets.append(&mut user_tweets);
    }
    tweets.sort_by_key(|tweet| tweet.created_at.to_string());

    let mut new_tweets: Vec<TweetResponse> = vec![];

    for tweet in tweets {
        if let None = feed
            .entries
            .iter()
            .find(|e| e.id == tweet.id_str.parse::<u64>().unwrap())
        {
            new_tweets.push(tweet);
        }
    }

    log::info!("Found {} new tweets to add to the feed", new_tweets.len());
    if !new_tweets.is_empty() {
        let num_of_new_tweets = new_tweets.len();
        for new_tweet in new_tweets.into_iter() {
            log::info!("----------------------------------");
            log::info!("Created At: {}", new_tweet.created_at);
            log::info!("Text: {}", new_tweet.full_text);
            log::info!("Link: {FX_TWITTER}{}", new_tweet.permalink);
            log::info!("----------------------------------");
            let entry = store::FeedEntry {
                id: new_tweet.id_str.parse::<u64>().unwrap(),
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
                destination: feed.destination.to_string(),
            })
            .await?;
        }

        store.save_feed(&feed)?;
        log::info!("Feed updated with {num_of_new_tweets} new entries");
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
