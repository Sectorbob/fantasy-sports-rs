pub fn setup_logger() {
    tracing_subscriber::fmt().init();
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    if let Err(err) = dotenv::dotenv() {
        eprintln!("Failed to load .env file: {}", err);
    }
    setup_logger();

    Ok(Config {
        discord_token: match std::env::var("DISCORD_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(
                "DISCORD_TOKEN environment variable is not set. Please set it to your bot token."
                    .into(),
            );
            }
        },
        guild_id: match std::env::var("DISCORD_GUILD_ID") {
            Ok(id) => id
                .parse::<u64>()
                .map_err(|_| "DISCORD_GUILD_ID must be a valid u64")?,
            Err(_) => {
                return Err(
                    "DISCORD_GUILD_ID environment variable is not set. Please set it to your guild ID."
                        .into(),
                );
            }
        },
        channel_id: match std::env::var("DISCORD_CHANNEL_ID") {
            Ok(id) => id
                .parse::<u64>()
                .map_err(|_| "DISCORD_CHANNEL_ID must be a valid u64")?,
            Err(_) => {
                return Err(
                    "DISCORD_CHANNEL_ID environment variable is not set. Please set it to your channel ID."
                        .into(),
                );
            }
        },
        twitter_subs: match std::env::var("TWITTER_SUBS") {
            Ok(subs) => subs.split(',').map(String::from).collect(),
            Err(_) => {
                return Err(
                    "TWITTER_SUBS environment variable is not set. Please set it to a comma delimited list."
                        .into(),
                );
            }
        },
        twitter_cookie: match std::env::var("TWITTER_COOKIE") {
            Ok(cookie) => cookie,
            Err(_) => {
                return Err(
                    "TWITTER_COOKIE environment variable is not set. Please set it.".into(),
                );
            }
        },
        yt_api_key: match std::env::var("YT_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return Err("YT_API_KEY environment variable is not set. Please set it.".into());
            }
        },
    })
}

pub struct Config {
    pub discord_token: String,
    pub guild_id: u64,
    pub channel_id: u64,
    pub twitter_subs: Vec<String>,
    pub twitter_cookie: String,
    pub yt_api_key: String,
}
