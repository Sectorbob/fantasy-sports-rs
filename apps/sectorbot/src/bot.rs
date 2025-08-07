use std::sync::Arc;

use serenity::all::{CacheHttp, ChannelId, Context, CreateMessage, EventHandler, GuildId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

use sleeper_fantasy_rs::custom::Standings;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::config::Config;
use crate::ecr;
use crate::ecr::service::{ECRError, LEAGUE_ID, SPORT};
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
    pub ecr_service: ecr::service::Service,
}
impl Handler {
    async fn process_health_command(&self, ctx: &Context, msg: &Message) -> Result<(), ECRError> {
        let message = format!(
            "Youtube credits consumed: {}",
            self.yt_client.credits_consumed()
        );
        send_as_code(&ctx, &msg, message).await;
        Ok(())
    }

    async fn process_keepables_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        let keepables_ctx = self
            .ecr_service
            .fetch_keepable_context(LEAGUE_ID, &SPORT)
            .await?;
        let keepables_report = self.ecr_service.get_keepables(&keepables_ctx)?;
        send_as_code(&ctx, &msg, keepables_report.text_report()).await;
        Ok(())
    }

    async fn process_draft_order_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        let standings = Standings::from_sleeper(LEAGUE_ID).await?;
        let order = standings.randomize_bucketed_draft_order();
        let text_report = Standings::visualize_draft_order(order).await?;
        send_as_code(&ctx, &msg, text_report).await;
        Ok(())
    }
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
            return;
        }
        if msg.content == "!health" {
            if let Err(err) = self.process_health_command(&ctx, &msg).await {
                let_em_know(&ctx.http, &msg, err).await;
            }
            return;
        }
        if msg.content == "!keepables" {
            if let Err(err) = self.process_keepables_command(&ctx, &msg).await {
                let_em_know(&ctx.http, &msg, err).await;
            }
            return;
        }
        if msg.content == "!draftorder" {
            if let Err(err) = self.process_draft_order_command(&ctx, &msg).await {
                let_em_know(&ctx.http, &msg, err).await;
            }
            return;
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

                let content = format!("{}\n{}", feed_msg.entry.title, feed_msg.entry.link);

                let message = CreateMessage::new().content(content);

                // // add a timestamp embed for youtube videos
                // if feed_msg.entry.id.platform == Some(store::Platform::Youtube) {
                //     message = message.add_embed(
                //         CreateEmbed::new()
                //             .timestamp(feed_msg.entry.source_ts)
                //             .footer(CreateEmbedFooter::new(format!("Published at: {}", feed_msg.entry.source_ts))),
                //     );
                // }

                // if !feed_msg.entry.link.is_empty() {
                //     message = message.add_embed(
                //         CreateEmbed::new()
                //             .title(&feed_msg.entry.title)
                //             // .description(&feed_msg.entry.title)
                //             .url(&feed_msg.entry.link),
                //     );
                // }

                // Add author if available
                // if let Some(author) = &feed_msg.entry.author {
                //     message = message.add_embed(
                //         CreateEmbed::new()
                //             .author(CreateEmbedAuthor::new(author.name.clone()).url(author.url.clone())),
                //     );
                // }

                if let Err(e) = channel_id.send_message(&ctx_clone.http, message).await {
                    log::error!("Failed to send message: {:?}", e);
                }
            }
        });
    }
}

const MAX_MESSAGE_LEN: usize = 2000;
const CODE_MAX_CONTENT_LEN: usize = MAX_MESSAGE_LEN - 8;
const CODE_PFX: &str = "```\n";
const CODE_SFX: &str = "\n```";

async fn send_as_code<T: Into<String>>(ctx: &Context, msg: &Message, text: T) {
    let mut text: String = text.into();

    text.truncate(CODE_MAX_CONTENT_LEN);

    // let active_message = String::from(CODE_PFX);

    // for line in text.lines() {

    // }

    if let Err(why) = msg
        .channel_id
        .say(&ctx.http, format!("{CODE_PFX}{text}{CODE_SFX}"))
        .await
    {
        log::error!("Error sending message: {why:?}");
    }
}

async fn let_em_know(http: &impl CacheHttp, msg: &Message, err: ECRError) {
    log::error!("failed: {err}");
    if let Err(why) = msg.channel_id.say(http, err.to_string()).await {
        log::error!("Error sending message: {why:?}");
    }
}

// mod tests {
//     use std::collections::VecDeque;

//     use serenity::all::{GetMessages, Http};

//     use super::*;
//     use crate::yt::YoutubeClient;

//     #[tokio::test]
//     async fn test_send_discord_message() {
//         crate::config::load_config().expect("Failed to load config");
//         // let nfl_news_channel_id: u64 = 2067967944778588584;
//         let giants_news_channel_id: u64 = 7100738766260628716;
//         let channel_id = ChannelId::new(giants_news_channel_id);
//         let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
//         let http = Http::new(discord_token.as_str());

//         let messages = channel_id
//             .messages(&http, GetMessages::new().limit(1))//.after(1471143525422999373))
//             .await
//             .expect("Failed to fetch messages");
//         log::info!("Fetched {} messages from channel {}", messages.len(), giants_news_channel_id);
//         for message in &messages {
//             log::info!("Message: {:#?}", message);
//         }
//         panic!("Test completed, check logs for messages");
//     }
// }
