use serenity::all::{
    CacheHttp, ChannelId, Context, CreateEmbed, CreateMessage, EventHandler, GuildId, Member,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use sleeper_fantasy_rs::custom::Standings;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::config::Config;
use crate::ecr;
use crate::ecr::service::{ECRError, LEAGUE_ID, SPORT};
use crate::store::{self, ExternalId, FeedEvent, League};
use crate::yt::YoutubeClient;

const TEST_SERVER_ID: u64 = 0;
const SECTORBOB_DISCORD_USER_ID: u64 = 87326629677516917;

pub struct AppContext {
    pub config: Config,
    pub store: Arc<store::DataStore>,
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
    // pub sleeper_cli: sleeper_fantasy_rs::Client,
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

    async fn process_info_command(&self, ctx: &Context, msg: &Message) -> Result<(), ECRError> {
        let mut text = String::new();
        let na = String::from("N/A");

        // let guild = msg.guild(&ctx.cache);

        let channel_id = msg.channel_id.to_string();
        let channel_name = msg
            .channel_id
            .name(&ctx.http)
            .await
            .map_or(na.clone(), |name| name);

        let guild_id = &msg.guild_id;
        let league_meta_data = guild_id
            .map(|guild_id| {
                self.ecr_service
                    .fetch_league_metadata(guild_id.to_string())
                    .inspect_err(|err| {
                        log::warn!("can't find league for guild: {err}");
                    })
                    .ok()
            })
            .flatten();

        let server_id = guild_id.map(|id| id.to_string()).unwrap_or(na.clone());
        let server_name = msg
            .guild(&ctx.cache)
            .map(|guild| guild.name.clone())
            .unwrap_or(na.clone());

        let league_id = league_meta_data
            .as_ref()
            .map(League::current_league_id)
            .flatten();
        let league = match &league_id {
            Some(league_id) => self
                .ecr_service
                .fetch_external_league_info(league_id.clone())
                .await
                .inspect_err(|err| log::warn!("external league not found: {err}"))
                .ok(),
            None => None,
        };
        let league_owners = match league_id {
            Some(league_id) => self
                .ecr_service
                .fetch_external_league_owners(league_id.clone())
                .await
                .inspect_err(|err| log::warn!("external league users not found: {err}"))
                .ok()
                .unwrap_or_default(),
            None => vec![],
        };
        let league_id = league
            .as_ref()
            .map(|league| league.league_id.clone())
            .unwrap_or(na.clone());
        let league_name = league
            .as_ref()
            .map(|league| league.name.clone())
            .unwrap_or(na.clone());
        let league_size = league
            .map(|league| league.settings.num_teams.to_string())
            .unwrap_or(na.clone());
        let league_owners_string = if league_owners.is_empty() {
            na.clone()
        } else {
            let mut s = league_owners.len().to_string();
            for owner in &league_owners {
                s.push_str(format!("\n - {} ({})", owner.display_name, owner.user_id).as_str());
            }
            s
        };

        let mut guild_members = vec![];
        if let Some(guild_id) = guild_id {
            log::info!("fetching guild info for {}", guild_id);
            if let Some(guild) = ctx.cache.guild(guild_id) {
                log::info!("fetching guild members for {}", guild.name);
                for (_member_id, member) in guild
                    .members
                    .iter()
                    .filter(|(_user_id, member)| !member.user.bot)
                {
                    log::debug!("non-bot Member Name: {}", member.user.name);
                    guild_members.push(member.clone());
                    // Access other member properties as needed
                }
            }
        }
        let guild_members_string = if guild_members.is_empty() {
            na.clone()
        } else {
            let mut s = guild_members.len().to_string();
            for member in &guild_members {
                let member_name = member.display_name();
                let member_user_id = member.user.id.to_string();
                let mapped_to = league_meta_data
                    .as_ref()
                    .map(|l| l.current_user_id(member_user_id.as_str()))
                    .flatten()
                    .map_or(String::from("UNMAPPED"), |id| id.to_string());
                s.push_str(format!("\n - {member_name} ({member_user_id}) {mapped_to}").as_str());
            }
            s
        };

        let league_history = league_meta_data.map_or(vec![], |league| {
            let mut seasons: Vec<&i32> = league.league_id_map.keys().collect();
            seasons.sort();
            seasons
                .into_iter()
                .map(|season| {
                    format!(
                        "{season}: {}",
                        league
                            .league_id_map
                            .get(season)
                            .map_or(format!("{season}: N/A"), ExternalId::to_string)
                    )
                })
                .collect()
        });
        text.push_str(format!("Server ID:    {server_id}\n").as_str());
        text.push_str(format!("Server Name:  {server_name}\n").as_str());
        text.push_str(format!("Channel ID:   {channel_id}\n").as_str());
        text.push_str(format!("Channel Name: {channel_name}\n").as_str());
        text.push_str(format!("League ID:    {league_id}\n").as_str());
        text.push_str(format!("League Name:  {league_name}\n").as_str());
        text.push_str(format!("League Size:  {league_size}\n").as_str());
        text.push_str(format!("League Owners:  {league_owners_string}\n").as_str());
        text.push_str(format!("Server Members: {guild_members_string}\n").as_str());
        text.push_str(format!("History: {} seasons", league_history.len()).as_str());
        for season in league_history {
            text.push_str(format!("\n - {season}").as_str());
        }

        send_as_code(&ctx, &msg, text).await;
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

    async fn process_set_manager_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        log::error!("{:#?}", msg);

        let command_parts = msg.content.split_ascii_whitespace().collect::<Vec<_>>();
        if command_parts.len() != 3 {
            send_as_code(&ctx, &msg, "invalid command syntax").await;
            return Ok(());
        }
        let member_id = msg.mentions.first().ok_or(ECRError::Unknown(
            "need a mention for the discord user".to_string(),
        ))?;
        let external_id = ExternalId::try_from(command_parts[2])
            .map_err(|_| ECRError::Unknown("invalid external id format".to_string()))?;

        if let Some(guild_id) = msg.guild_id {
            self.ecr_service.save_user_mapping(
                guild_id.to_string(),
                member_id.id.to_string(),
                external_id,
            )?;
            self.ecr_service
                .synchronize_manager_role(guild_id, &ctx.http)
                .await?;
        }

        let mut response = String::new();
        response.push_str(format!("Content: {}", msg.content).as_str());

        send_as_code(&ctx, &msg, response).await;
        Ok(())
    }

    async fn match_to_external_league_user(
        &self,
        league_id: &ExternalId,
        member: &Member,
    ) -> Result<ExternalId, ECRError> {
        // perform a sleeper username lookup

        self.ecr_service
            .fetch_external_league_owners(league_id.clone())
            .await
            .inspect_err(|err| {
                log::warn!("unable to fetch league users: {err}");
                return;
            })
            .map(|users| {
                users
                    .into_iter()
                    .find(|user| user.display_name == member.user.display_name())
            })?
            .map(ExternalId::from)
            .ok_or_else(|| ECRError::NotFound(format!("did not match a user in sleeper league")))
    }

    fn is_permitted(ctx: &Context, msg: &Message) -> bool {
        msg.author.id.get() == SECTORBOB_DISCORD_USER_ID
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

        if msg.content.starts_with("!") && !Handler::is_permitted(&ctx, &msg) {
            // block it and delete the message
            log::warn!(
                "{user}, is not permitted to run commands",
                user = msg.author.display_name()
            );
            if let Err(why) = msg.delete(&ctx.http).await {
                log::warn!("failed to delete message: {why}");
            }
            return;
        }

        if msg.content == "!info" {
            if let Err(err) = self.process_info_command(&ctx, &msg).await {
                let_em_know(&ctx.http, &msg, err).await;
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

        if msg.content.starts_with("!set-manager ") {
            if let Err(err) = self.process_set_manager_command(&ctx, &msg).await {
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

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        // You'll need to enable the `GUILD_MEMBERS` privileged intent for this event to fire.

        // Get the Guild ID if possible
        let guild_id = new_member.guild_id;
        let member_user_id = new_member.user.id.to_string();

        // lookup league
        let league = match self.ecr_service.fetch_league_metadata(guild_id.to_string()) {
            Ok(league) => league,
            Err(err) => {
                log::warn!("unable to find league for guild: {err}");
                return;
            }
        };

        // check to see if we can do an easy user_id map
        if let Some(league_id) = league.current_league_id() {
            if let Some(_current_league_user_id) = league.current_user_id(member_user_id.as_str()) {
                // already mapped
            } else {
                // try to match display name
                if let Ok(external_user_id) = self
                    .match_to_external_league_user(&league_id, &new_member)
                    .await
                {
                    // found a match, now update the DataStore
                    if let Err(err) = self.ecr_service.save_user_mapping(
                        guild_id.to_string(),
                        member_user_id,
                        external_user_id,
                    ) {
                        log::error!("failed to set user mapping in datastore: {err}");
                    }
                }
            }
        }

        // // Fetch the guild to get a relevant channel
        // if let Some(guild) = guild_id.to_guild_cached(&ctx.cache) {
        //     // Find a channel to send the welcome message
        //     // (e.g., a "welcome" channel or the first text channel)
        //     if let Some(channel_id) = guild.system_channel_id {
        //         // or a specific channel ID
        //         let user_name = &new_member.user.name;

        //         // Create an embed for a more visually appealing welcome message
        //         let embed = CreateEmbed::new()
        //             .title(format!("Welcome to {}!", guild.name))
        //             .description(format!("Glad to have you here, {}!", user_name))
        //             .field("Rules", "Make sure to read the rules!", false)
        //             .thumbnail(new_member.user.avatar_url().unwrap_or_default())
        //             .color(0x00FF00); // Green color

        //         let builder = CreateMessage::new().embed(embed);

        //         if let Err(why) = channel_id.send_message(&ctx.http, builder).await {
        //             println!("Error sending welcome message: {:?}", why);
        //         }
        //     } else {
        //         println!(
        //             "Could not find a channel to send welcome message in guild: {}",
        //             guild.name
        //         );
        //     }
        // } else {
        //     println!("Could not fetch guild from cache for ID: {}", guild_id);
        // }
    }
}

const MAX_MESSAGE_LEN: usize = 2000;
const CODE_MAX_CONTENT_LEN: usize = MAX_MESSAGE_LEN - 8;
const CODE_PFX: &str = "```\n";
const CODE_SFX: &str = "```";

async fn send_as_code<T: Into<String>>(ctx: &Context, msg: &Message, text: T) {
    let text: String = text.into();

    let mut line_iterator = text.lines();

    let mut messages_to_send: Vec<String> = vec![];
    let mut line = line_iterator.next();
    while let Some(line_text) = line {
        let mut active_message = String::from("");
        let mut space_left = CODE_MAX_CONTENT_LEN;
        let mut line_text = line_text;
        let mut lines = 0;
        while line.is_some() && space_left > line_text.len() {
            active_message.push_str(line_text);
            active_message.push('\n');
            lines = lines + 1;
            space_left = space_left - line_text.len() - 1;
            line = line_iterator.next();
            if let Some(tmp) = line.clone() {
                line_text = tmp;
            }
        }
        // send
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("lines in message: {lines}");
            log::debug!("characters in message: {}", active_message.len());
            log::debug!("message: {active_message}");
        }
        messages_to_send.push(active_message);
    }
    for text in messages_to_send {
        if let Err(why) = msg
            .channel_id
            .say(&ctx.http, format!("{CODE_PFX}{text}{CODE_SFX}"))
            .await
        {
            log::error!("Error sending message: {why:?}");
            break;
        }
    }
}

async fn let_em_know(http: &impl CacheHttp, msg: &Message, err: ECRError) {
    log::error!("failed: {err}");
    if let Err(why) = msg.channel_id.say(http, err.to_string()).await {
        log::error!("Error sending message: {why:?}");
    }
}

mod tests {
    use std::collections::{HashMap, VecDeque};

    use serenity::all::{
        Colour, CreateEmbedAuthor, CreateEmbedFooter, EditMember, GetMessages, Http,
        MessageBuilder, Permissions, Role, RoleId, UserId,
    };
    use serenity::builder::EditRole;
    use serenity::futures::StreamExt;

    use super::*;
    use crate::ecr::service::Service;
    use crate::store::{DataStore, Platform};
    use crate::twitter::{TweetResponse, syndication};
    use crate::yt::YoutubeClient;

    #[tokio::test]
    async fn test_send_discord_message() {
        let config = crate::config::load_config().expect("Failed to load config");
        // let nfl_news_channel_id: u64 = 2067967944778588584;
        let giants_news_channel_id: u64 = 7100738766260628716;
        let channel_id = ChannelId::new(giants_news_channel_id);
        let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
        let http = Http::new(discord_token.as_str());

        let messages = channel_id
            .messages(
                &http,
                GetMessages::new().limit(1).after(8604299660934911324),
            )
            .await
            .expect("Failed to fetch messages");
        log::info!(
            "Fetched {} messages from channel {}",
            messages.len(),
            giants_news_channel_id
        );
        for message in &messages {
            log::info!("Message: {:#?}", message);
        }
        // panic!("Test completed, check logs for messages");
        let syndication_client = syndication::Client::new();

        let tweet = syndication_client
            .fetch_tweet(3446382222382249349, config.twitter_cookie.clone())
            .await
            .expect("failed to get tweet");
        println!("Tweet: {tweet:#?}");
        channel_id
            .send_message(
                &http,
                CreateMessage::new().add_embeds(discord_embed_for_tweet(tweet)),
            )
            .await
            .expect("failed to send message");
        // println!("Tweet: {tweet:#?}");
        assert!(false);
    }

    const footer_twitter_icon: &str = "https://assets.fxembed.com/logos/fxtwitter-pride64.png";
    const twitter_color_hex: u32 = 6513919;

    fn discord_embed_for_tweet(tweet: TweetResponse) -> Vec<CreateEmbed> {
        let mut embeds = vec![];
        embeds.push(
            CreateEmbed::new()
                .author(
                    CreateEmbedAuthor::new(&tweet.user.name)
                        .url("https://x.com/BobbySkinner_/status/3446382222382249349")
                        .icon_url("https://x.com/BobbySkinner_/status/3446382222382249349"),
                )
                .timestamp(tweet.created_at)
                .footer(CreateEmbedFooter::new("Twitter").icon_url(footer_twitter_icon))
                .colour(Colour::new(twitter_color_hex))
                .description("Thought this went well\\. Excited to see what Jordan does with it\\.\n\u{fe00}\u{fe00}\n\u{fe00}\u{fe00}See you all at Giants\\-Jets practice tomorrow\\. Recap to follow\n\n> **[Quoting](https://x.com/JordanRaanan/status/3713477757564328062) Jordan Raanan \\([@JordanRaanan](https://x.com/JordanRaanan)\\)**\n> \u{fe00}\n> Impressive Giants Rookies \\| Breaking Big Blue w/ Jordan Raanan\\. Better viewed on Youtube [youtube.com/live/YNvRkrzh2fo?feature=share](https://youtube.com/live/YNvRkrzh2fo?feature=share) [x.com/i/broadcasts/1mnxeNyLyjbKX](https://x.com/i/broadcasts/1mnxeNyLyjbKX)\n\n**[❤\u{fe0f}](https://x.com/intent/like?tweet_id=3446382222382249349) 2\u{2002}👁\u{fe0f} 57\u{2002}**")
                .url("https://fxtwitter.com/BobbySkinner_/status/3446382222382249349"),
        );

        embeds
        // MessageBuilder::new().emb.build()
    }

    // #[tokio::test]
    // async fn test_create_role() {
    //     crate::config::load_config().expect("Failed to load config");
    //     // let nfl_news_channel_id: u64 = 2067967944778588584;
    //     let giants_news_channel_id: u64 = 7100738766260628716;
    //     let channel_id = ChannelId::new(giants_news_channel_id);
    //     let discord_token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    //     let http = Http::new(discord_token.as_str());
    //     let store = Arc::new(DataStore::new());
    //     let svc: Service = Service::new(&store);

    //     let sectorbobs_football_lounge_guild_id = 6828159027854455257;

    //     let guild_id = GuildId::new(sectorbobs_football_lounge_guild_id);

    //     let mut existing_roles = guild_id.roles(&http).await.expect("can't get roles");
    //     let custom_roles = svc
    //         .create_necessary_roles(guild_id, &http)
    //         .await
    //         .expect("can't setup custom roles");

    //     svc.synchronize_manager_role(guild_id, &http)
    //         .await
    //         .expect("unable to syncropnize manager role");

    //     // let new_role = guild_id.create_role(&http, role_builder).await;

    //     // match new_role {
    //     //     Ok(role) => println!("Successfully created role: {}", role.name),
    //     //     Err(why) => println!("Error creating role: {:?}", why),
    //     // }

    //     panic!("Test completed, check logs for messages");
    // }
}
