use fantasy_sports_sleeper::custom::Standings;
use serenity::all::{
    CacheHttp, ChannelId, Colour, Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    CreateMessage, EventHandler, Member,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::config::Config;
use crate::ecr::service::{ECRError, SPORT};
use crate::store::{self, ExternalId, FeedEvent, League, Platform};
use crate::twitter::syndication;
use crate::yt::{self, YoutubeClient};
use crate::{ecr, twitter};

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
    /// Sleeper league for `!keepables` / `!draft`; those commands are disabled when `None`.
    pub sleeper_league_id: Option<String>,
    /// The only user allowed to run `!` commands; all are refused when `None`.
    pub admin_discord_user_id: Option<u64>,
    pub yt_client: Arc<YoutubeClient>,
    pub events_receiver: Arc<Mutex<Receiver<FeedEvent>>>,
    pub ecr_service: ecr::service::Service,
    // pub sleeper_cli: fantasy_sports_sleeper::Client,
}
impl Handler {
    fn league_id(&self) -> Result<&str, ECRError> {
        self.sleeper_league_id
            .as_deref()
            .ok_or_else(|| ECRError::NotFound("SLEEPER_LEAGUE_ID is not configured".to_string()))
    }

    async fn process_health_command(&self, ctx: &Context, msg: &Message) -> Result<(), ECRError> {
        let message = format!(
            "Youtube credits consumed: {}",
            self.yt_client.credits_consumed()
        );
        send_as_code(ctx, msg, message).await;
        Ok(())
    }

    async fn process_info_command(&self, ctx: &Context, msg: &Message) -> Result<(), ECRError> {
        let mut text = String::new();
        let na = String::from("N/A");

        // let guild = msg.guild(&ctx.cache);

        let channel_id = msg.channel_id.to_string();
        let channel_name = msg.channel_id.name(&ctx.http).await.unwrap_or(na.clone());

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
            .and_then(League::current_league_id);
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
                    .and_then(|l| l.current_user_id(member_user_id.as_str()))
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

        send_as_code(ctx, msg, text).await;
        Ok(())
    }

    async fn process_keepables_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        let keepables_ctx = self
            .ecr_service
            .fetch_keepable_context(self.league_id()?, &SPORT)
            .await?;
        let keepables_report = self.ecr_service.get_keepables(&keepables_ctx)?;
        send_as_code(ctx, msg, keepables_report.text_report()).await;
        Ok(())
    }

    async fn process_draft_order_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        let league_id = self.league_id()?;
        let standings = Standings::from_sleeper(league_id).await?;
        let order = standings.randomize_bucketed_draft_order();
        let text_report = Standings::visualize_draft_order(league_id, order).await?;
        send_as_code(ctx, msg, text_report).await;
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
            send_as_code(ctx, msg, "invalid command syntax").await;
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

        send_as_code(ctx, msg, response).await;
        Ok(())
    }

    async fn process_set_league_command(
        &self,
        ctx: &Context,
        msg: &Message,
    ) -> Result<(), ECRError> {
        log::error!("{:#?}", msg);

        let command_parts = msg.content.split_ascii_whitespace().collect::<Vec<_>>();
        if command_parts.len() != 3 {
            send_as_code(ctx, msg, "invalid command syntax").await;
            return Ok(());
        }
        let season: i32 = command_parts[1]
            .parse()
            .map_err(|_| ECRError::Unknown("season (arg 1) must be an int value".to_string()))?;
        let external_id = ExternalId::try_from(command_parts[2])
            .map_err(|_| ECRError::Unknown("invalid external id format".to_string()))?;
        if !external_id.is_fantasy_platform() {
            return Err(ECRError::Unknown(
                "external league id platform is not value".to_string(),
            ));
        }

        if let Some(guild_id) = msg.guild_id {
            self.ecr_service
                .save_league_mapping(guild_id.to_string(), season, external_id)?;
        }

        let mut response = String::new();
        response.push_str(format!("Content: {}", msg.content).as_str());

        send_as_code(ctx, msg, response).await;
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
            })
            .map(|users| {
                users
                    .into_iter()
                    .find(|user| user.display_name == member.user.display_name())
            })?
            .map(ExternalId::from)
            .ok_or_else(|| ECRError::NotFound("did not match a user in sleeper league".to_string()))
    }

    fn is_permitted(&self, msg: &Message) -> bool {
        self.admin_discord_user_id
            .is_some_and(|admin| msg.author.id.get() == admin)
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

        if msg.content.starts_with("!") && !self.is_permitted(&msg) {
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
        if msg.content.starts_with("!set-league ") {
            if let Err(err) = self.process_set_league_command(&ctx, &msg).await {
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
                    let guild_id = guild.id;
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
            let syndication_client = syndication::Client::new();
            let mut rx = feed_rx.lock().await;
            while let Some(feed_msg) = rx.recv().await {
                let channel_id = ChannelId::new(feed_msg.destination.id.parse().unwrap());

                let mut message: Option<CreateMessage> = None;
                if let Some(platform) = &feed_msg.entry.id.platform
                    && platform == &Platform::Twitter
                    && let Ok(tweet_id) = feed_msg.entry.id.id.parse::<u64>()
                    && let Ok(tweet) = syndication_client.fetch_tweet(tweet_id).await
                {
                    message = Some(CreateMessage::new().add_embed(discord_embed_for_tweet(&tweet)))
                }
                // default formatting if all the optional values are included
                if message.is_none()
                    && feed_msg.entry.author_name.is_some()
                    && feed_msg.entry.author_id.is_some()
                {
                    let mut embed_author =
                        CreateEmbedAuthor::new(feed_msg.entry.author_name.unwrap_or_default())
                            .url(feed_msg.entry.link.clone());
                    if let Some(author_icon) = feed_msg.entry.author_icon_url {
                        embed_author = embed_author.icon_url(author_icon.as_str());
                    }

                    let color = if let Some(platform) = &feed_msg.entry.id.platform {
                        match platform {
                            store::Platform::Twitter => Some(Colour::new(TWITTER_COLOR_HEX)),
                            store::Platform::Youtube => Some(Colour::new(YOUTUBE_COLOR_HEX)),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let footer = if let Some(platform) = &feed_msg.entry.id.platform {
                        match platform {
                            store::Platform::Twitter => Some(
                                CreateEmbedFooter::new("Twitter").icon_url(FOOTER_TWITTER_ICON),
                            ),
                            store::Platform::Youtube => Some(
                                CreateEmbedFooter::new("Youtube").icon_url(FOOTER_YOUTUBE_ICON),
                            ),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    let mut embed = CreateEmbed::new()
                        .author(embed_author)
                        .timestamp(feed_msg.entry.source_ts)
                        .description(feed_msg.entry.content)
                        .title(feed_msg.entry.title.clone())
                        .url(feed_msg.entry.link.clone());
                    if let Some(color) = color {
                        embed = embed.color(color);
                    }
                    if let Some(footer) = footer {
                        embed = embed.footer(footer);
                    }

                    message = Some(CreateMessage::new().embed(embed));
                }

                // default formatting
                if message.is_none() {
                    let content = format!("{}\n{}", feed_msg.entry.title, feed_msg.entry.link);
                    message = Some(CreateMessage::new().content(content));
                }

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
                if let Some(message) = message
                    && let Err(e) = channel_id.send_message(&ctx_clone.http, message).await
                {
                    log::error!("Failed to send message: {:?}", e);
                }
            }
        });
    }

    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
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
const FOOTER_TWITTER_ICON: &str = "https://cdn-icons-png.flaticon.com/64/145/145812.png"; //"https://assets.fxembed.com/logos/fxtwitter-pride64.png";
const FOOTER_YOUTUBE_ICON: &str = "https://cdn-icons-png.flaticon.com/64/174/174883.png";
const TWITTER_COLOR_HEX: u32 = 6513919;
const YOUTUBE_COLOR_HEX: u32 = 16711680;

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
            lines += 1;
            space_left = space_left - line_text.len() - 1;
            line = line_iterator.next();
            if let Some(tmp) = line {
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

fn discord_embed_for_youtube(
    video: &yt::Item,
    channel_icons: &HashMap<String, String>,
) -> Option<CreateEmbed> {
    let channel_name = video
        .snippet
        .as_ref()
        .and_then(|snippet| snippet.channel_title.clone())?;
    let channel_id = video
        .snippet
        .as_ref()
        .and_then(|snippet| snippet.channel_id.clone())?;
    let channel_icon = channel_icons.get(&channel_id).cloned();
    let link = video.link()?;
    let title = video.title();
    let description = encode_text(video.description().as_str());
    let thumbnail_image_url = video
        .snippet
        .as_ref()
        .map(|s| s.thumbnails.medium.url.clone());
    let timestamp = video.snippet.as_ref().map(|snippet| snippet.published_at)?;
    let mut embed_author = CreateEmbedAuthor::new(channel_name).url(link.as_str());
    if let Some(channel_icon) = channel_icon {
        embed_author = embed_author.icon_url(channel_icon.as_str());
    }

    let mut embed = CreateEmbed::new()
        .author(embed_author)
        .timestamp(timestamp)
        .footer(CreateEmbedFooter::new("Youtube").icon_url(FOOTER_YOUTUBE_ICON))
        .colour(Colour::new(YOUTUBE_COLOR_HEX))
        .description(description)
        .title(title)
        .url(link.as_str());

    if let Some(thumbnail_image_url) = thumbnail_image_url {
        embed = embed.image(thumbnail_image_url.as_str());
    }

    Some(embed)
}

/// Retweet and like counts as intent links, using only the counts the payload provides.
fn engagement_line(tweet: &twitter::TweetResponse) -> Option<String> {
    let mut parts = vec![];
    if let Some(retweets) = tweet.retweet_count {
        parts.push(format!(
            "[🔁](https://x.com/intent/retweet?tweet_id={}) {retweets}",
            tweet.id_str
        ));
    }
    if let Some(likes) = tweet.favorite_count {
        parts.push(format!(
            "[❤\u{fe0f}](https://x.com/intent/like?tweet_id={}) {likes}",
            tweet.id_str
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("\n\n**{}\u{2002}**", parts.join("\u{2002}")))
    }
}

fn discord_embed_for_tweet(tweet: &twitter::TweetResponse) -> CreateEmbed {
    let link = format!("https://x.com{}", tweet.link());
    let mut description = encode_text(tweet.text().as_str());

    if let Some(quoted) = &tweet.quoted_tweet {
        description.push_str("\n\n> **[Quoting](https://x.com");
        description.push_str(&quoted.link());
        description.push_str(") ");
        description.push_str(&quoted.user.name);
        description.push_str(
            format!(
                " \\([@{screen_name}](https://x.com/{screen_name})\\)**\n> \u{fe00}\n> ",
                screen_name = quoted.user.screen_name
            )
            .as_str(),
        );
        description.push_str(encode_text(&quoted.text()).as_str());
    }
    if let Some(stats) = engagement_line(tweet) {
        description.push_str(&stats);
    }

    let mut embed = CreateEmbed::new()
        .author(
            CreateEmbedAuthor::new(format!("{} (@{})", tweet.user.name, tweet.user.screen_name))
                .url(link.as_str())
                .icon_url(tweet.user.profile_image_url_https.as_str()),
        )
        .timestamp(tweet.created_at)
        .footer(CreateEmbedFooter::new("Twitter").icon_url(FOOTER_TWITTER_ICON))
        .colour(Colour::new(TWITTER_COLOR_HEX))
        .description(description)
        .url(link.as_str());

    embed = insert_from_media_details(embed, tweet);
    if let Some(quoted) = &tweet.quoted_tweet {
        embed = insert_from_media_details(embed, quoted);
    }

    embed
}

fn encode_text(s: &str) -> String {
    let mut s = regex::escape(s);
    s = s.replace("\n", "\n\u{fe00}\u{fe00}");
    s = s.replace("\\.", "."); // undo the period escape
    s
}

fn insert_from_media_details(
    mut embed: CreateEmbed,
    tweet: &twitter::TweetResponse,
) -> CreateEmbed {
    if let Some(media_details) = &tweet.media_details {
        for media in media_details {
            match media.r#type {
                twitter::MediaType::Photo => {
                    embed = embed.image(media.media_url_https.as_str());
                }
                twitter::MediaType::Video => {
                    // TODO: figure out how I want to do this. discord doesn't allow me to add video embeds
                    // embed.(media.url);
                }
            }
        }
    }
    embed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::twitter::TweetResponse;
    use crate::yt;
    use serenity::all::Embed;

    #[tokio::test]
    async fn test_tweet_to_embed() {
        let tweet_id: u64 = 1800000000000000101;
        let tweet_str =
            tokio::fs::read_to_string(format!("testdata/twitter/tweet_{tweet_id}.json"))
                .await
                .expect("failed to load test data file");
        let tweet: TweetResponse =
            serde_json::from_str(tweet_str.as_str()).expect("failed to deserialize tweet json");
        let embed = to_embed(discord_embed_for_tweet(&tweet));
        let expected = to_embed(CreateEmbed::new()
            .url(format!("https://x.com/SampleScribe/status/{tweet_id}")) // I am using x.com instead of fxtwitter.com
            .author(
                CreateEmbedAuthor::new("Sam Sample (@SampleScribe)")
                    .url(format!("https://x.com/SampleScribe/status/{tweet_id}"))
                    .icon_url("https://pbs.example.invalid/profile_images/1000000001/avatar_normal.jpg"),
            ).image("https://pbs.example.invalid/media/sample001.jpg")
            .footer(CreateEmbedFooter::new("FxTwitter").icon_url("https://assets.fxembed.com/logos/fxtwitter-pride64.png"))
            .colour(Colour(6513919))
            .description("If rookie Riley Rookie hits for the Rovers \\-\\- and not just in a down the line kinda way \\-\\- extend the OL coach immediately.\n\u{fe00}\u{fe00}The Rovers have been desperate for a Day 3 OL draft hit since Pat Anchor \\(Lee Backup I guess\\).\n\n> **[Quoting](https://x.com/alexquote/status/1800000000000000099) Alex Quote \\([@alexquote](https://x.com/alexquote)\\)**\n> \u{fe00}\n> \u{2066}@RileyRookie\u{2069} has reason to smile. He continues to handle his business and held his own at right tackle with the first team https://t.example.invalid/pic001\n\n**[🔁](https://x.com/intent/retweet?tweet_id=1800000000000000101) 7\u{2002}[❤\u{fe0f}](https://x.com/intent/like?tweet_id=1800000000000000101) 212\u{2002}**"),
        );
        debug_assert_eq!(expected.author, embed.author, "embed author is not right");
        debug_assert_eq!(expected.kind, embed.kind, "embed type is not right");
        debug_assert_eq!(expected.url, embed.url, "embed url is not right");
        debug_assert_eq!(expected.image, embed.image, "embed image is not right");
        debug_assert_eq!(
            expected.description, embed.description,
            "embed description is not right"
        );
    }

    fn tweet_with_counts(counts: &str) -> TweetResponse {
        serde_json::from_str(&format!(
            r#"{{"id_str":"42","text":"hi","lang":"en","created_at":"2025-01-15T17:13:22.000Z",
                "user":{{"id_str":"1","name":"Sam Sample","screen_name":"SampleScribe","profile_image_url_https":"https://pbs.example.invalid/a.jpg"}}{counts}}}"#
        ))
        .expect("valid tweet json")
    }

    #[test]
    fn test_engagement_line_uses_the_counts_in_the_payload() {
        let both = engagement_line(&tweet_with_counts(
            r#","retweet_count":3,"favorite_count":15"#,
        ))
        .unwrap();
        assert!(both.contains("tweet_id=42) 3"), "{both}");
        assert!(both.contains("like?tweet_id=42) 15"), "{both}");

        let likes_only = engagement_line(&tweet_with_counts(r#","favorite_count":15"#)).unwrap();
        assert!(!likes_only.contains("retweet"), "{likes_only}");
        assert!(likes_only.contains(") 15"), "{likes_only}");

        assert_eq!(None, engagement_line(&tweet_with_counts("")));
    }

    #[test]
    fn test_plain_tweet_embed_also_shows_counts() {
        let embed = to_embed(discord_embed_for_tweet(&tweet_with_counts(
            r#","favorite_count":15"#,
        )));
        assert!(embed.description.unwrap().ends_with(") 15\u{2002}**"));
    }

    fn to_embed(create_embed: CreateEmbed) -> Embed {
        let json = serde_json::to_string(&create_embed).expect("fail");
        serde_json::from_str(json.as_str()).expect("fail")
    }

    #[tokio::test]
    async fn test_youtube_embed_uses_the_configured_channel_icon() {
        let json = tokio::fs::read_to_string("testdata/yt/list_playlist_items.json")
            .await
            .unwrap();
        let yt::Response::PlaylistItemListResponse { items, .. } =
            serde_json::from_str(&json).unwrap()
        else {
            panic!("expected a playlist item response");
        };
        let icons = HashMap::from([(
            "UCexampleRoverBanter0001".to_string(),
            "https://img.example.invalid/icon.png".to_string(),
        )]);
        let embed = to_embed(discord_embed_for_youtube(&items[0], &icons).unwrap());
        assert_eq!(
            Some("https://img.example.invalid/icon.png".to_string()),
            embed.author.and_then(|a| a.icon_url)
        );
    }

    #[tokio::test]
    async fn test_youtube_to_embed() {
        let youtube_playlist_videos_str =
            tokio::fs::read_to_string("testdata/yt/list_playlist_items.json".to_string())
                .await
                .expect("failed to load test data file");
        let yt_videos_response: yt::Response =
            serde_json::from_str(youtube_playlist_videos_str.as_str())
                .expect("failed to deserialize list channels json");

        let videos = match yt_videos_response {
            yt::Response::PlaylistItemListResponse { items, .. } => Some(items),
            _ => None,
        }
        .expect("no videos loaded");

        let embed = to_embed(
            discord_embed_for_youtube(&videos[0], &HashMap::new()).expect("no embed created"),
        );
        let expected = to_embed(CreateEmbed::new()
            .url("https://www.youtube.com/watch?v=exampleVid01".to_string()) // I am using x.com instead of fxtwitter.com
            .author(
                CreateEmbedAuthor::new("Rover Banter")
                    .url("https://www.youtube.com/watch?v=exampleVid01".to_string()),
                // no icon: the channel is not in the configured icon map
            ).image("https://img.example.invalid/exampleVid01/medium.jpg")
            .footer(CreateEmbedFooter::new("Youtube").icon_url(FOOTER_YOUTUBE_ICON))
            .colour(Colour(6513919))
            .title("Rovers Camp: rookie delivers + big play in practice")
            .description("Sam and Alex break down the last two Rovers practices and a deep ball. That and a whole lot more!\n\u{fe00}\u{fe00}\n\u{fe00}\u{fe00}\\#Rovers \\#football \\#camp")
        );
        debug_assert_eq!(expected.author, embed.author, "embed author is not right");
        debug_assert_eq!(expected.kind, embed.kind, "embed type is not right");
        debug_assert_eq!(expected.url, embed.url, "embed url is not right");
        debug_assert_eq!(expected.image, embed.image, "embed image is not right");
        debug_assert_eq!(
            expected.description, embed.description,
            "embed description is not right"
        );
    }
}
