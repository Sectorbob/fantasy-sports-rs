use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

// Deserialize the JSON response into a struct
#[derive(Debug, Deserialize)]
pub struct TweetResponse {
    pub id_str: String,
    pub text: String,
    pub user: User,
    #[serde(deserialize_with = "parse_custom_datetime")]
    pub created_at: DateTime<FixedOffset>,
    pub full_text: String, // "Kurt Warner described this as the \"best throw in Super Bowl history\" and he's RIGHT\n\nIf the ball has any less velocity, it's incomplete. An inch left? Incomplete. An inch right? Interception\n\nCapped off a playoff run where he threw 11 TDs to 1 INT and nearly 300 YPG\n\n#GiantsPride",
    pub lang: String,
    pub permalink: String, //"/DanSchneierNFL/status/1220109293313282051",
                           // Add more fields as needed
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
    // Add more fields as needed
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineResponse {
    pub props: TimelineProps,
    pub page: String,
    pub query: HashMap<String, String>,
    pub build_id: String,
    pub asset_prefix: String,
    pub is_fallback: bool,
    pub gssp: bool,
    pub custom_server: bool,
    pub script_loader: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineProps {
    pub page_props: PageProps,
    #[serde(rename = "__N_SSP")]
    pub nssp: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProps {
    // contextProvider: {},
    pub lang: String,
    pub max_height: Option<u32>,
    pub show_header: bool,
    pub hide_border: bool,
    pub hide_footer: bool,
    pub hide_scroll_bar: bool,
    pub transparent: bool,
    pub timeline: Timeline,
    #[serde(rename = "latest_tweet_id")]
    pub latest_tweet_id: String,
    pub header_props: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Timeline {
    pub entries: Vec<TimelineEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct TimelineEntry {
    #[serde(rename = "type")]
    pub _type: String,
    pub entry_id: String,
    pub sort_index: String,
    pub content: TimelineContent,
}

#[derive(Debug, Deserialize)]
pub struct TimelineContent {
    pub tweet: Option<TweetResponse>,
}

fn parse_custom_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_str(&s, "%a %b %d %H:%M:%S %z %Y").map_err(Error::custom)
}
