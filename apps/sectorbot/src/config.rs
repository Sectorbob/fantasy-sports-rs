use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::store::Feed;

const ENV_HIDE_LOG_TS: &str = "HIDE_LOG_TS";
const ENV_YT_API_KEY: &str = "YT_API_KEY";
const ENV_TWITTER_COOKIE: &str = "TWITTER_COOKIE";
const ENV_DISCORD_TOKEN: &str = "DISCORD_TOKEN";

const ENV_SLEEPER_LEAGUE_ID: &str = "SLEEPER_LEAGUE_ID";
const ENV_ADMIN_DISCORD_USER_ID: &str = "ADMIN_DISCORD_USER_ID";
const ENV_FEEDS_FILE: &str = "FEEDS_FILE";
const ENV_PUBLIC_BASE_URL: &str = "PUBLIC_BASE_URL";

const DEFAULT_HTTP_SERVER_PORT: u16 = 8080;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("Failed to load .env file: {}", err);
    }
    setup_logger();

    let http_server_port = match std::env::var("HTTP_SERVER_PORT") {
        Ok(port_str) => port_str.parse::<u16>().unwrap_or(DEFAULT_HTTP_SERVER_PORT),
        Err(_) => DEFAULT_HTTP_SERVER_PORT,
    };

    Ok(Config {
        http_server_port,
        public_base_url: std::env::var(ENV_PUBLIC_BASE_URL)
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| format!("http://localhost:{http_server_port}")),
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
        sleeper_league_id: std::env::var(ENV_SLEEPER_LEAGUE_ID)
            .ok()
            .filter(|v| !v.is_empty()),
        admin_discord_user_id: match std::env::var(ENV_ADMIN_DISCORD_USER_ID) {
            Ok(id) if !id.is_empty() => {
                Some(id.parse::<u64>().map_err(|e| {
                    format!("{ENV_ADMIN_DISCORD_USER_ID} is not a valid user id: {e}")
                })?)
            }
            _ => None,
        },
        feeds_file: std::env::var_os(ENV_FEEDS_FILE).map(PathBuf::from),
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
        if hide_ts.eq_ignore_ascii_case("TRUE") {
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
    /// Externally visible base url, used for links in the RSS feeds.
    pub public_base_url: String,
    pub discord_token: String,
    pub twitter_cookie: String,
    pub yt_api_key: String,
    /// Sleeper league used by the `!keepables` and `!draft` commands; they are disabled when unset.
    pub sleeper_league_id: Option<String>,
    /// The only Discord user allowed to run `!` commands; commands are disabled when unset.
    pub admin_discord_user_id: Option<u64>,
    /// Optional JSON file ([`FeedsFile`]) whose feeds are added to the data store at startup.
    pub feeds_file: Option<PathBuf>,
}

/// Contents of the optional feeds file, see `feeds.example.json`.
#[derive(Debug, Default, Deserialize)]
pub struct FeedsFile {
    #[serde(default)]
    pub feeds: Vec<Feed>,
}

pub fn load_feeds_file(path: &Path) -> Result<FeedsFile, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Ok(serde_json::from_str(&text)
        .map_err(|e| format!("invalid feeds file {}: {e}", path.display()))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_feeds_file_parses() {
        let file = load_feeds_file(Path::new("feeds.example.json"))
            .expect("example feeds file should parse");
        assert_eq!(2, file.feeds.len());
        assert_eq!("Rovers News", file.feeds[0].name);
        assert!(file.feeds.iter().all(|f| !f.sources.is_empty()));
    }

    #[test]
    fn missing_feeds_file_is_a_readable_error() {
        let err = load_feeds_file(Path::new("does-not-exist.json"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("does-not-exist.json"), "{err}");
    }
}
