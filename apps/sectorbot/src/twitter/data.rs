use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

// Deserialize the JSON response into a struct
#[derive(Debug, Deserialize, Clone)]
pub struct TweetResponse {
    pub id_str: String,
    pub(crate) text: String,
    pub user: User,
    #[serde(deserialize_with = "parse_custom_datetime")]
    pub created_at: DateTime<FixedOffset>,
    pub(crate) full_text: Option<String>, // full tweet text, when the payload provides it
    pub lang: String,
    pub favorite_count: Option<u64>,
    pub retweet_count: Option<u64>,
    pub(crate) permalink: Option<String>, //"/SampleScribe/status/1800000000000000050",
    pub quoted_tweet: Option<Box<TweetResponse>>,
    #[serde(rename = "mediaDetails")]
    pub media_details: Option<Vec<MediaDetails>>,
    // Add more fields as needed
}
impl TweetResponse {
    pub fn link(&self) -> String {
        if let Some(permalink) = &self.permalink {
            permalink.clone()
        } else {
            format!(
                "/{username}/status/{tweet_id}",
                username = self.user.screen_name,
                tweet_id = self.id_str
            )
        }
    }
    pub fn text(&self) -> String {
        if let Some(full_text) = &self.full_text {
            full_text.clone()
        } else {
            self.text.clone()
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id_str: String,
    pub name: String,
    pub screen_name: String,
    pub profile_image_url_https: String,
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
impl TimelineResponse {
    pub fn tweets(&self) -> Vec<&TweetResponse> {
        self.props
            .page_props
            .timeline
            .entries
            .iter()
            .filter_map(|te| te.content.tweet.as_ref().map(|t| t))
            .collect()
    }
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

#[derive(Debug, Deserialize, Clone)]
pub struct MediaDetails {
    pub display_url: String,
    pub expanded_url: String,
    // pub ext_media_availability: struct,
    pub media_url_https: String,
    pub r#type: MediaType,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Photo,
    Video,
}

const CUSTOM_DT_FMT: &str = "%a %b %d %H:%M:%S %z %Y";

fn parse_custom_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let s = String::deserialize(deserializer)?;

    if let Ok(ts) = DateTime::parse_from_rfc3339(&s) {
        // try rfc3339
        Ok(ts)
    } else if let Ok(ts) = DateTime::parse_from_str(&s, CUSTOM_DT_FMT) {
        // try custom
        Ok(ts)
    } else {
        // error
        Err(Error::custom(format!(
            "could not parse: {s}. supported formats: [rfc3339, {CUSTOM_DT_FMT}]"
        )))
    }
}

#[cfg(test)]
mod data_tests {
    use chrono::{FixedOffset, TimeZone};
    use serde_assert::{Deserializer as SerdeAssertDeserializer, Token};

    use crate::twitter::data::parse_custom_datetime;

    #[test]
    fn test_parse_custom_date_ts_from_timeline() {
        let cases = vec![
            (
                "Sun Jan 01 21:01:16 +0000 2023",
                FixedOffset::east_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2023, 1, 1, 21, 1, 16)
                    .unwrap(),
            ),
            (
                "2025-08-12T02:06:25.000Z",
                FixedOffset::east_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2025, 8, 12, 2, 6, 25)
                    .unwrap(),
            ),
        ];

        for test_case in cases {
            let mut deserializer =
                SerdeAssertDeserializer::builder(vec![Token::Str(test_case.0.to_string())]).build();

            debug_assert_eq!(
                parse_custom_datetime(&mut deserializer),
                Ok(test_case.1),
                "date parse is wrong"
            );
        }
    }
}
