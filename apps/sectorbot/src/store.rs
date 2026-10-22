use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::US::Pacific;
use core::fmt;
use fantasy_sports_sleeper::User as SleeperUser;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::str::FromStr;

const DEFAULT_DESTINATION_PLATFORM: Platform = Platform::Discord;
const DEFAULT_SOURCE_PLATFORM: Platform = Platform::Twitter;

#[allow(dead_code)]
#[derive(Clone)]
pub struct DataStore {
    feeds: sled::Db,
    quotas: sled::Db,
    leagues: sled::Db,
}
impl DataStore {
    pub fn new() -> Self {
        let feeds = sled::open("data_store/feeds").expect("Failed to open data store");
        let quotas = sled::open("data_store/quotas").expect("Failed to open quotas store");
        let leagues = sled::open("data_store/leagues").expect("Failed to open leagues store");
        DataStore {
            feeds,
            quotas,
            leagues,
        }
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

    pub fn save_feed(&self, feed: &Feed) -> Result<(), DataStoreError> {
        let data = serde_json::to_vec(feed)?;
        self.feeds.insert(feed.id.to_string(), data)?;
        self.feeds.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_feed(&self, id: i64) -> Result<(), DataStoreError> {
        let _old_entry = self.feeds.remove(id.to_string())?;
        self.feeds.flush()?;
        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>, DataStoreError> {
        DataStore::list_collection(&self.feeds)
    }
    // Add methods to interact with the store as needed

    #[allow(dead_code)]
    pub fn get_quota(
        &self,
        platform: &Platform,
        on_date: Option<NaiveDate>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let key = QuotaKey::new(platform.clone(), on_date.unwrap_or_else(today_pacific));
        match self.quotas.get(String::from(key))? {
            Some(value) => {
                let quota: PlatformQuota = serde_json::from_slice(&value)?;
                Ok(quota.quota)
            }
            None => Ok(0), // Default quota if not set
        }
    }

    #[allow(dead_code)]
    pub fn set_quota(
        &self,
        platform: &Platform,
        on_date: Option<NaiveDate>,
        quota: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = QuotaKey::new(platform.clone(), on_date.unwrap_or_else(today_pacific));
        let quota_data = PlatformQuota {
            date: key.date,
            platform: key.platform.clone(),
            quota,
        };
        let data = serde_json::to_vec(&quota_data)?;
        self.quotas.insert(String::from(key), data)?;
        self.quotas.flush()?;
        Ok(())
    }

    pub fn get_league<T: Into<String>>(&self, guild_id: T) -> Result<League, DataStoreError> {
        let guild_id: String = guild_id.into();
        let bytes = self
            .leagues
            .get(guild_id.as_str())?
            .ok_or(DataStoreError::ElementNotFound(guild_id))?;
        Ok(serde_json::from_slice::<League>(&bytes)?)
    }

    pub fn save_league(&self, league: &League) -> Result<(), DataStoreError> {
        let data = serde_json::to_vec(league)?;
        self.leagues.insert(&league.guild_id, data)?;
        self.leagues.flush()?;
        Ok(())
    }

    pub fn list_leagues(&self) -> Result<Vec<League>, DataStoreError> {
        DataStore::list_collection(&self.leagues)
    }

    fn list_collection<T: for<'de> Deserialize<'de>>(
        db: &sled::Db,
    ) -> Result<Vec<T>, DataStoreError> {
        let mut items = Vec::new();
        for (_key, value) in db.iter().flatten() {
            items.push(serde_json::from_slice::<T>(&value)?);
        }
        Ok(items)
    }

    #[allow(dead_code)]
    pub fn set_league_season_mapping<G: Into<String>>(
        &self,
        guild_id: G,
        season: i32,
        external_league_id: ExternalId,
    ) -> Result<(), DataStoreError> {
        let guild_id = guild_id.into();
        let mut league = self.get_league(guild_id)?;
        league.league_id_map.insert(season, external_league_id);
        self.save_league(&league)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_league_user_mapping<G: Into<String>, M: Into<String>>(
        &self,
        guild_id: G,
        member_id: M,
        external_user_id: ExternalId,
    ) -> Result<(), DataStoreError> {
        let platform = &external_user_id.platform.ok_or(DataStoreError::DevError(
            "cannot set league user id mapping without external id platform".into(),
        ))?;

        let guild_id = guild_id.into();
        let member_id = member_id.into();

        let mut league = self.get_league(guild_id)?;
        let changed = if let Some(entry) = league.member_map.get_mut(&member_id) {
            if entry.get(platform).is_some() {
                // overwrite it
                let existing_value = entry.insert(platform.clone(), external_user_id.id.clone());
                existing_value.is_none_or(|v| v != external_user_id.id)
            } else {
                // insert it
                entry.insert(platform.clone(), external_user_id.id);
                true
            }
        } else {
            // create new entry for member
            league.member_map.insert(
                member_id,
                HashMap::from([(platform.clone(), external_user_id.id)]),
            );
            true
        };

        if changed {
            self.save_league(&league)?;
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DataStoreError {
    #[error("element not found {0}")]
    ElementNotFound(String),

    #[error("failed to fetch from sleeper {0}")]
    SledError(#[from] sled::Error),

    #[error("failed to deserialize item: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("failed to deserialize item: {0}")]
    DevError(String),
}

fn today_pacific() -> NaiveDate {
    // Get today's date in Pacific Time
    let now_utc: DateTime<Utc> = Utc::now();
    let pacific_now: DateTime<chrono_tz::Tz> = Pacific.from_utc_datetime(&now_utc.naive_utc());
    pacific_now.date_naive()
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
    pub author_icon_url: Option<String>,
    pub author_name: Option<String>,
    pub author_id: Option<String>,
    pub embed_image_url: Option<String>,
}

#[allow(dead_code)]
pub struct FeedEvent {
    pub entry: FeedEntry,
    pub feed_id: u64,
    pub destination: ExternalId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalId {
    pub platform: Option<Platform>,
    pub id: String,
}
impl ExternalId {
    pub fn new(platform: Platform, id: String) -> Self {
        ExternalId {
            platform: Some(platform),
            id,
        }
    }
    fn parse<T: Into<String>>(v: T) -> Result<Self, ParseExternalIdError> {
        let v = v.into();
        if v.contains(':') {
            let parts: Vec<&str> = v.split(':').collect();
            if parts.len() == 2 {
                // Reconstruct the string or store the parts as needed
                return Ok(ExternalId {
                    platform: Some(
                        Platform::from_str(parts[0]).map_err(ParseExternalIdError::from)?,
                    ),
                    id: parts[1].to_string(),
                });
            } else if parts.len() > 2 {
                return Err(ParseExternalIdError::from(
                    "expecting a string with two segments delinted by a :",
                ));
            }
        }
        Ok(ExternalId {
            platform: None,
            id: v.to_string(),
        })
    }

    fn get_platform(self) -> Option<Platform> {
        self.platform.clone()
    }
    pub fn is_fantasy_platform(&self) -> bool {
        self.platform.as_ref().is_some_and(|p| match p {
            Platform::Sleeper => true,
            Platform::Yahoo => true,
            _ => false,
        })
    }
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
                ExternalId::try_from(v).map_err(|err| {
                    E::invalid_value(
                        serde::de::Unexpected::Str(err.to_string().as_str()),
                        &"a valid platform in lowercase",
                    )
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
        serializer.serialize_str(&self.to_string())
    }
}
impl fmt::Display for ExternalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(platform) = &self.platform {
            write!(f, "{}:{}", platform, self.id)
        } else {
            write!(f, "{}", self.id)
        }
    }
}
impl TryFrom<&str> for ExternalId {
    type Error = ParseExternalIdError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ExternalId::parse(value)
    }
}
impl TryFrom<String> for ExternalId {
    type Error = ParseExternalIdError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ExternalId::parse(value)
    }
}
impl From<SleeperUser> for ExternalId {
    fn from(value: SleeperUser) -> Self {
        ExternalId {
            platform: Some(Platform::Sleeper),
            id: value.user_id.clone(),
        }
    }
}
impl From<&SleeperUser> for ExternalId {
    fn from(value: &SleeperUser) -> Self {
        ExternalId {
            platform: Some(Platform::Sleeper),
            id: value.user_id.clone(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct ParseExternalIdError {
    reason: String,
}
impl ParseExternalIdError {
    pub fn from<T: Into<String>>(reason: T) -> Self {
        ParseExternalIdError {
            reason: reason.into(),
        }
    }
}
impl fmt::Display for ParseExternalIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid external id: {}", self.reason)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Discord,
    Sleeper,
    Twitter,
    Yahoo,
    Youtube,
}
impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "discord" => Ok(Platform::Discord),
            "twitter" => Ok(Platform::Twitter),
            "youtube" => Ok(Platform::Youtube),
            "sleeper" => Ok(Platform::Sleeper),
            "yahoo" => Ok(Platform::Yahoo),
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
            Platform::Sleeper => write!(f, "sleeper"),
            Platform::Yahoo => write!(f, "yahoo"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformQuota {
    pub date: NaiveDate,
    pub platform: Platform,
    pub quota: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotaKey {
    pub platform: Platform,
    pub date: NaiveDate,
}
impl QuotaKey {
    pub fn new(platform: Platform, date: NaiveDate) -> Self {
        QuotaKey { platform, date }
    }
}
impl From<QuotaKey> for String {
    fn from(key: QuotaKey) -> Self {
        format!("{}:{}", key.platform, key.date)
    }
}
impl From<&QuotaKey> for String {
    fn from(key: &QuotaKey) -> Self {
        format!("{}:{}", key.platform, key.date)
    }
}
impl From<Platform> for String {
    fn from(platform: Platform) -> Self {
        format!("{}:{}", platform, today_pacific())
    }
}
impl Serialize for QuotaKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.platform, self.date))
    }
}
impl<'de> Deserialize<'de> for QuotaKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::custom("Invalid QuotaKey format"));
        }
        let platform = Platform::from_str(parts[0]).map_err(serde::de::Error::custom)?;
        let date =
            NaiveDate::parse_from_str(parts[1], "%Y-%m-%d").map_err(serde::de::Error::custom)?;
        Ok(QuotaKey { platform, date })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct League {
    pub guild_id: String,
    pub current_season: i32,
    pub league_id_map: HashMap<i32, ExternalId>,
    pub member_map: HashMap<String, HashMap<Platform, String>>,
}
impl League {
    pub fn current_league_id(&self) -> Option<ExternalId> {
        self.league_id_map.get(&self.current_season).cloned()
    }
    pub fn current_user_id<T: Into<String>>(&self, member_id: T) -> Option<ExternalId> {
        self.current_league_id()
            .and_then(ExternalId::get_platform)
            .and_then(|current_league_platform: Platform| {
                self.member_map
                    .get(&member_id.into())
                    .and_then(|m| m.get(&current_league_platform))
                    .map(|id| ExternalId::new(current_league_platform, id.to_string()))
            })
    }
}

#[cfg(test)]
#[allow(unused_imports)]
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
        "id":1800000000000000050,
        "title":"I have to be the first person in Sample Takes history to be on the good end of one of these right??\nLFG #Rovers !!!",
        "link":"https://fxtwitter.com/SampleScribe/status/1800000000000000050",
        "source_ts":"2023-01-01T21:01:16Z",
        "content":"I have to be the first person in Sample Takes history to be on the good end of one of these right??\nLFG #Rovers !!!",
        "published_at":null
        }
        "#;
        let entry: crate::store::FeedEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id.platform, None);
        assert_eq!(entry.id.id, "1800000000000000050");
        assert_eq!(
            entry.title,
            "I have to be the first person in Sample Takes history to be on the good end of one of these right??\nLFG #Rovers !!!"
        );
        assert_eq!(
            entry.link,
            "https://fxtwitter.com/SampleScribe/status/1800000000000000050"
        );
        assert_eq!(
            entry.content,
            "I have to be the first person in Sample Takes history to be on the good end of one of these right??\nLFG #Rovers !!!"
        );
    }
}
