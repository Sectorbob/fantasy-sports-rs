use chrono::{DateTime, Utc};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

const DEFAULT_DESTINATION_PLATFORM: Platform = Platform::Discord;
const DEFAULT_SOURCE_PLATFORM: Platform = Platform::Twitter;

#[derive(Clone)]
pub struct DataStore {
    feeds: sled::Db,
}
impl DataStore {
    pub fn new() -> Self {
        let feeds = sled::open("data_store/feeds").expect("Failed to open data store");
        DataStore { feeds }
    }

    pub fn get_feed(&self, id: u64) -> Option<Feed> {
        self.feeds
            .get(id.to_string())
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .map(|feed: Feed| {
                // Ensure the feed has a valid destination platform
                let mut feed = if feed.destination.platform.is_none() {
                    let mut feed = feed;
                    feed.destination.platform = Some(DEFAULT_DESTINATION_PLATFORM.clone());
                    feed
                } else {
                    feed
                };
                // Ensure the feed sources have a value (DEFAULT To twitter)
                feed.sources.iter_mut().for_each(|source| {
                    if source.platform.is_none() {
                        source.platform = Some(DEFAULT_SOURCE_PLATFORM.clone());
                    }
                });
                // Ensure the feed entries have a value (DEFAULT To twitter)
                feed.entries.iter_mut().for_each(|entry| {
                    if entry.id.platform.is_none() {
                        entry.id.platform = Some(DEFAULT_SOURCE_PLATFORM.clone());
                    }
                });
                feed
            })
    }

    pub fn save_feed(&self, feed: &Feed) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(feed)?;
        self.feeds.insert(feed.id.to_string(), data)?;
        self.feeds.flush()?;
        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>, Box<dyn std::error::Error>> {
        let mut feeds = Vec::new();
        for item in self.feeds.iter() {
            if let Ok((_key, value)) = item {
                match serde_json::from_slice::<Feed>(&value) {
                    Ok(feed) => feeds.push(feed),
                    Err(e) => {
                        log::error!("Failed to deserialize feed: {e}. value: {}", String::from_utf8_lossy(&value));
                        continue;
                    }
                }
            }
        }
        Ok(feeds)
    }
    // Add methods to interact with the store as needed
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub sources: Vec<ExternalId>,
    pub last_updated: Option<String>,
    pub destination: ExternalId,
    pub pool_interval: u64,
    pub quiet_time: String,
    pub entries: Vec<FeedEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeedEntry {
    pub id: ExternalId,
    pub title: String,
    pub link: String,
    pub source_ts: DateTime<Utc>,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
}

pub struct FeedEvent {
    pub entry: FeedEntry,
    pub feed_id: u64,
    pub destination: ExternalId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalId {
    pub platform: Option<Platform>,
    pub id: String,
}

impl<'de> Deserialize<'de> for ExternalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FeedDestinationVisitor;

        impl<'de> Visitor<'de> for FeedDestinationVisitor {
            type Value = ExternalId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string with or without a colon delimiter")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                // Here, you would implement the logic to handle the optional colon
                // during deserialization.
                //
                // Example:
                if v.contains(':') {
                    let parts: Vec<&str> = v.split(':').collect();
                    if parts.len() == 2 {
                        // Reconstruct the string or store the parts as needed
                        return Ok(ExternalId {
                            platform: match Platform::from_str(parts[0]) {
                                Ok(platform) => Some(platform),
                                Err(err) => {
                                    return Err(E::invalid_value(
                                        serde::de::Unexpected::Str(err.as_str()),
                                        &"a valid platform in lowercase",
                                    ));
                                }
                            },
                            id: parts[1].to_string(),
                        });
                    } else if parts.len() > 2 {
                        return Err(E::custom("Invalid format for FeedDestination"));
                    }
                }
                Ok(ExternalId {
                    platform: None,
                    id: v.to_string(),
                })
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ExternalId {
                    platform: None,
                    id: v.to_string(),
                })
            }

            // Implement other `visit_*` methods as needed depending on the expected input format
        }

        deserializer.deserialize_any(FeedDestinationVisitor)
    }
}

impl Serialize for ExternalId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(platform) = &self.platform {
            // Serialize as "platform:id"
            serializer.serialize_str(&format!("{}:{}", platform.to_string(), self.id))
        } else {
            // If platform is None, just serialize the id
            serializer.serialize_str(&self.id)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Discord,
    Twitter,
    Youtube,
}
impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "discord" => Ok(Platform::Discord),
            "twitter" => Ok(Platform::Twitter),
            "youtube" => Ok(Platform::Youtube),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}
impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Discord => write!(f, "discord"),
            Platform::Twitter => write!(f, "twitter"),
            Platform::Youtube => write!(f, "youtube"),
        }
    }
}

mod tests {
    use crate::store::{ExternalId, Platform};

    #[test]
    fn test_external_id_deserialization() {
        let json = r#""twitter:1234567890""#;
        let external_id: ExternalId = serde_json::from_str(json).unwrap();
        assert_eq!(external_id.platform, Some(Platform::Twitter));
        assert_eq!(external_id.id, "1234567890");

        let json_no_platform = r#""1234567890""#;
        let external_id_no_platform: ExternalId = serde_json::from_str(json_no_platform).unwrap();
        assert!(external_id_no_platform.platform.is_none());
        assert_eq!(external_id_no_platform.id, "1234567890");
    }

    #[test]
    fn test_external_id_serialization() {
        let external_id = ExternalId {
            platform: Some(Platform::Discord),
            id: "asd789".to_string(),
        };
        let json = serde_json::to_string(&external_id).unwrap();
        assert_eq!(json, r#""discord:asd789""#);
        let external_id_no_platform = ExternalId {
            platform: None,
            id: "asdf".to_string(),
        };
        let json_no_platform = serde_json::to_string(&external_id_no_platform).unwrap();
        assert_eq!(json_no_platform, r#""asdf""#);
    }

    #[test]
    fn test_feed_entry_deserilization() {
        let json = r#"
        {
        "id":7301820310883384255,
        "title":"I have to be the first person in Freezing Cold Takes history to be on the good end of one of these right??\nLFG #Giants !!!",
        "link":"https://fxtwitter.com/DanSchneierNFL/status/7301820310883384255",
        "source_ts":"2023-01-01T21:01:16Z",
        "content":"I have to be the first person in Freezing Cold Takes history to be on the good end of one of these right??\nLFG #Giants !!!",
        "published_at":null
        }
        "#;
        let entry: crate::store::FeedEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id.platform, None);
        assert_eq!(entry.id.id, "7301820310883384255");
        assert_eq!(entry.title, "I have to be the first person in Freezing Cold Takes history to be on the good end of one of these right??\nLFG #Giants !!!");
        assert_eq!(entry.link, "https://fxtwitter.com/DanSchneierNFL/status/7301820310883384255");
        assert_eq!(entry.content, "I have to be the first person in Freezing Cold Takes history to be on the good end of one of these right??\nLFG #Giants !!!");
    }
}
