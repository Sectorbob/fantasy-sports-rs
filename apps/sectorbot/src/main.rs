mod config;
mod tweeter;

use std::error::Error;
use std::fmt::Display;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::config::Config;
use crate::tweeter::TweetResponse;

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
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::setup_logger();
    let config = config::load_config()?;
    // setup_discord_bot(&config).await?;
    do_twitter_fetch(&config.twitter_subs, &config.twitter_cookie).await?;
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
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        log::error!("Client error: {why:?}");
    }
    Ok(())
}

const FX_TWITTER: &str = "https://fxtwitter.com";

async fn do_twitter_fetch(
    subs: &Vec<String>,
    cookie: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    // let twitter_username = "giantinsider";
    // let subs = vec![
    //     "DanSchneierNFL",
    //     "giantinsider",
    //     "nickfalato",
    //     "SharpFootball",
    // ];

    let mut tweets: Vec<TweetResponse> = vec![];
    for twitter_username in subs {
        let mut user_tweets = fetch_raw_tweets(twitter_username, cookie).await?;
        println!(
            "Twitter Timeline for {}. {} entries",
            twitter_username,
            user_tweets.len()
        );
        tweets.append(&mut user_tweets);
    }
    tweets.sort_by_key(|tweet| tweet.created_at.to_string());

    for tweet in tweets {
        println!("----------------------------------");
        println!("Created At: {}", tweet.created_at);
        println!("Text: {}", tweet.full_text);
        println!("Link: {FX_TWITTER}{}", tweet.permalink);
        println!("----------------------------------");
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
