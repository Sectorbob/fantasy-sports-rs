use std::sync::Arc;

use serenity::all::{ChannelId, Context, EventHandler, GuildId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::config::Config;
use crate::store::{self, FeedEvent};
use crate::yt::YoutubeClient;

pub struct AppContext {
    pub config: Config,
    pub store: store::DataStore,
    pub new_tweets_sender: Sender<FeedEvent>,
    pub yt_client: Arc<crate::yt::YoutubeClient>,
}
impl Drop for AppContext {
    fn drop(&mut self) {}
}

pub struct Handler {
    pub yt_client: Arc<YoutubeClient>,
    pub events_receiver: Arc<Mutex<Receiver<FeedEvent>>>,
}

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
        if msg.content == "!health" {
            let message = format!("Youtube credits consumed: {}", self.yt_client.credits_consumed());
            if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
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
        log::debug!("Context: {:?}", ctx);

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
                            log::debug!("Channels in guild {}: {:?}", guild_id, channels);
                            for (channel_id, channel) in channels {
                                log::debug!("Channel ID: {}, Name: {}", channel_id, channel.name);
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

        let ctx_clone = ctx.clone();
        let feed_rx = Arc::clone(&self.events_receiver);

        // Consumer: receive feed events and send to Discord
        log::info!("Starting event receiver task...");
        tokio::spawn(async move {
            let mut rx = feed_rx.lock().await;
            while let Some(feed_msg) = rx.recv().await {
                let channel_id = ChannelId::new(feed_msg.destination.id.parse().unwrap());
                if let Err(e) = channel_id.say(&ctx_clone.http, &feed_msg.entry.link).await {
                    log::error!("Failed to send message: {:?}", e);
                }
            }
        });
    }
}
