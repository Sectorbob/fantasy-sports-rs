const ENV_HIDE_LOG_TS: &str = "HIDE_LOG_TS";
const ENV_YT_API_KEY: &str = "YT_API_KEY";
const ENV_TWITTER_COOKIE: &str = "TWITTER_COOKIE";
const ENV_DISCORD_TOKEN: &str = "DISCORD_TOKEN";

const DEFAULT_HTTP_SERVER_PORT: u16 = 8080;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    if let Err(err) = dotenv::dotenv() {
        eprintln!("Failed to load .env file: {}", err);
    }
    setup_logger();

    Ok(Config {
        http_server_port: match std::env::var("HTTP_SERVER_PORT") {
            Ok(port_str) => port_str
                .parse::<u16>()
                .unwrap_or(DEFAULT_HTTP_SERVER_PORT),
            Err(_) => DEFAULT_HTTP_SERVER_PORT,
        },
        discord_token: match std::env::var(ENV_DISCORD_TOKEN) {
            Ok(token) => token,
            Err(_) => {
                return Err(format!(
                "{ENV_DISCORD_TOKEN} environment variable is not set. Please set it to your bot token.")
                    .into(),
            );
            }
        },
        twitter_cookie: match std::env::var(ENV_TWITTER_COOKIE) {
            Ok(cookie) => cookie,
            Err(_) => {
                return Err(format!(
                    "{ENV_TWITTER_COOKIE} environment variable is not set. Please set it."
                )
                .into());
            }
        },
        yt_api_key: match std::env::var(ENV_YT_API_KEY) {
            Ok(key) => key,
            Err(_) => {
                return Err(format!(
                    "{ENV_YT_API_KEY} environment variable is not set. Please set it."
                )
                .into());
            }
        },
    })
}

fn setup_logger() {
    if let Some(hide_ts) = std::env::var_os(ENV_HIDE_LOG_TS) {
        if hide_ts.to_ascii_uppercase() == "TRUE" {
            tracing_subscriber::fmt().without_time().init();
        } else {
            tracing_subscriber::fmt().init();
        }
    } else {
        tracing_subscriber::fmt().init();
    }
}

pub struct Config {
    pub http_server_port: u16,
    pub discord_token: String,
    pub twitter_cookie: String,
    pub yt_api_key: String,
}
