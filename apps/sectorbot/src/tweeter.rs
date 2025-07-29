use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, FixedOffset};
use regex::Regex;
use serde::Deserialize;

fn f64_to_base36_string(n: f64, precision: usize) -> String {
    let int_part = n.trunc() as u64;
    let frac_part = n.fract();
    let int_str = format!("{:}", radix_fmt::radix(int_part, 36));
    let frac_precision = precision - int_str.len() + 1;
    let mut frac = frac_part;
    let mut frac_str = String::new();
    for _ in 0..frac_precision {
        frac *= 36.0;
        let digit = frac.trunc() as u8;
        frac_str.push(std::char::from_digit(digit as u32, 36).unwrap());

        frac -= digit as f64;
    }
    format!("{}.{}", int_str, frac_str)
}

fn get_token(id: u64) -> String {
    let value = (id as f64 / 1e15) * std::f64::consts::PI;
    log::debug!("Value: {}", value);
    // let base = 36u32; // 6 ** 2 = 36
    // (value.trunc() as u64).to_string_radix(36); // Convert f64 to base36-like string
    let mut s = f64_to_base36_string(value, 11); // Convert f64 to hex as a base36-like string
    log::debug!("Hex string: {}", s);
    // Remove all '0's and '.'s
    s.retain(|c| c != '0' && c != '.');
    s
}

pub async fn fetch_user_timeline<T: Display>(
    username: T,
    cookie: T,
) -> Result<TimelineResponse, Box<dyn std::error::Error>> {
    let token = get_token(username.to_string().len() as u64);
    log::debug!(
        "Fetching user timeline for: {} with token: {}",
        username,
        token
    );

    let client = reqwest::Client::new();
    let req = client
        .get(format!(
            "https://syndication.twitter.com/srv/timeline-profile/screen-name/{username}"
        ))
        .header("User-Agent", "curl/8.7.1")
        .header("Accept", "*/*")
        .header("Cookie", cookie.to_string()) // .query(&[("screen_name", username)/* , ("token", &token)*/])
        .build()?;
    log::debug!("Request URL: {}", req.url());
    let res = client.execute(req).await?;
    if res.status().is_success() {
        let html = res.text().await?;
        let re =
            Regex::new(r#"<script id="__NEXT_DATA__" type="application/json">([^>]*)</script>"#)
                .unwrap();

        if let Some(caps) = re.captures(html.as_str()) {
            let json_text = &caps[1];

            Ok(serde_json::from_str::<TimelineResponse>(json_text)?)
        } else {
            return Err("Failed to find JSON data in the response".into());
        }
    } else {
        Err(format!("Failed to fetch user timeline: {}", res.status()).into())
    }
}

pub async fn fetch_tweet(tweet_id: u64) -> Result<TweetResponse, Box<dyn std::error::Error>> {
    let token = get_token(tweet_id);
    log::debug!("Fetching tweet with ID: {} and token: {}", tweet_id, token);

    let client = reqwest::Client::new();
    let req = client
        .get("https://cdn.syndication.twimg.com/tweet-result")
        .header("User-Agent", "curl/8.7.1")
        .header("Accept", "application/json")
        .query(&[("id", &tweet_id.to_string()), ("token", &token.to_string())])
        .build()?;
    log::debug!("Request URL: {}", req.url());
    let res = client.execute(req).await?;
    if res.status().is_success() {
        Ok(res.json::<TweetResponse>().await?)
    } else {
        Err(format!("Failed to fetch tweet: {}", res.status()).into())
    }
}

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
    pub permalink: String, //"/DanSchneierNFL/status/3016845289181978600",
                           // Add more fields as needed
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
    // Add more fields as needed
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineProps {
    pub page_props: PageProps,
    #[serde(rename = "__N_SSP")]
    pub nssp: bool,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_black_box() {
        let tweet_id = "774794463037719";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        assert_eq!(token, "ggyb6ihoy75");

        let tweet_id = "7301820310883384255";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        assert_eq!(token, "3wgvt2kcol");
    }
}
