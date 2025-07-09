mod config;
mod store;
mod tweeter;

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use serenity::all::GuildId;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::config::Config;
use crate::store::Feed;
use crate::tweeter::TweetResponse;

struct AppContext {
    config: Config,
    logger_handle: flexi_logger::LoggerHandle,
    store: store::DataStore,
}
impl Drop for AppContext {
    fn drop(&mut self) {
        // Flush the logger when the context is dropped
        self.logger_handle.flush();
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                log::error!("Error sending message: {why:?}");
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just log what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
        log::info!("Context: {:?}", ctx);

        match ready.guilds.len() {
            0 => log::warn!("No guilds found!"),
            count => {
                log::info!("Connected to {} guilds.", count);
                for guild in &ready.guilds {
                    let guild_id = GuildId::from(guild.id);
                    log::info!("Guild ID: {}", guild.id);
                    if let Some(name) = guild_id.name(&ctx.cache) {
                        log::info!("Guild Name: {}", name);
                    }
                    match guild_id.channels(&ctx.http).await {
                        Ok(channels) => {
                            log::info!("Channels in guild {}: {:?}", guild_id, channels);
                            for (channel_id, channel) in channels {
                                log::info!("Channel ID: {}, Name: {}", channel_id, channel.name);
                            }
                        }
                        Err(why) => {
                            log::error!(
                                "Error fetching channels for guild {}: {:?}",
                                guild_id,
                                why
                            );
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger_handle = match config::setup_logger() {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to setup logger: {}", e);
            return Err(e);
        }
    };
    let store = store::DataStore::new();

    // store.save_feed(&Feed {
    //     id: 1,
    //     name: "Giants News".to_string(),
    //     description: "A curated list of Giants news sources".to_string(),
    //     sources: vec![
    //         "DanSchneierNFL".to_string(),
    //         "GiantInsider".to_string(),
    //         "CarlBanksGIII".to_string(),
    //         "nickfalato".to_string(),
    //         "BobbySkinner_".to_string(),
    //         // DanSchneierNFL,GiantInsider,SharpFootball,CarlBanksGIII,nickfalato,BobbySkinner_
    //         ],
    //     last_updated: None,
    //     destination: "7100738766260628716".to_string(),
    //     pool_interval: 60,
    //     quiet_time: "00:00-06:00".to_string(),
    //     entries: vec![],
    // })?;

    if let Some(feed) = store.get_feed(1) {
        // println!("Feed found: {:#?}", feed);
        println!("Feed found entries: {}", feed.entries.len());
    }

    log::info!("Data store initialized successfully!");
    let config = match config::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    let app_context = Arc::new(AppContext {
        config,
        logger_handle,
        store,
    });

    // match setup_discord_bot(&config).await {
    //     Ok(_) => log::info!("Discord bot setup successfully!"),
    //     Err(e) => {
    //         log::error!("Failed to setup Discord bot: {}", e);
    //         return Err(e);
    //     }
    // };
    do_twitter_fetch(&app_context).await?;
    Ok(())
}

async fn setup_discord_bot(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler)
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

async fn do_twitter_fetch(app_context: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let mut feed = app_context.store.get_feed(1).ok_or("Feed not found")?;

    let mut tweets: Vec<TweetResponse> = vec![];
    for twitter_username in feed.sources.iter() {
        let mut user_tweets =
            fetch_raw_tweets(twitter_username, &app_context.config.twitter_cookie).await?;
        println!(
            "Twitter Timeline for {}. {} entries",
            twitter_username,
            user_tweets.len()
        );
        tweets.append(&mut user_tweets);
    }
    tweets.sort_by_key(|tweet| tweet.created_at.to_string());

    let mut new_tweets: Vec<TweetResponse> = vec![];

    for tweet in tweets {
        // println!("----------------------------------");
        // println!("Created At: {}", tweet.created_at);
        // println!("Text: {}", tweet.full_text);
        // println!("Link: {FX_TWITTER}{}", tweet.permalink);
        // println!("----------------------------------");
        if let None = feed
            .entries
            .iter()
            .find(|e| e.id == tweet.id_str.parse::<u64>().unwrap())
        {
            new_tweets.push(tweet);
        }
    }

    log::info!("Found {} new tweets to add to the feed", new_tweets.len());

    for new_tweet in new_tweets.into_iter() {
        println!("----------------------------------");
        println!("Created At: {}", new_tweet.created_at);
        println!("Text: {}", new_tweet.full_text);
        println!("Link: {FX_TWITTER}{}", new_tweet.permalink);
        println!("----------------------------------");
        let entry = store::FeedEntry {
            id: new_tweet.id_str.parse::<u64>().unwrap(),
            title: new_tweet.full_text.clone(),
            link: format!("{FX_TWITTER}{}", new_tweet.permalink),
            source_ts: new_tweet.created_at.to_utc(),
            content: new_tweet.full_text.clone(),
            published_at: None, // You can set this to Some(Utc::now()) if you want
        };
        feed.entries.push(entry);
    }

    app_context.store.save_feed(&feed)?;
    log::info!("Feed updated with {} new entries", feed.entries.len());

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
