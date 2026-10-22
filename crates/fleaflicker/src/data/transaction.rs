use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClaimStatus {
    pub r#type: ClaimType,
    #[serde(rename = "lock_epoch_milli", with = "serde_epoch_millis")]
    pub lock_ts: DateTime<Utc>,
    #[serde(rename = "clear_epoch_milli", with = "serde_epoch_millis")]
    pub clear_ts: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimType {
    GameStart,
    RecentlyDropped,
    ContinuousPreseason,
    ContinuousInSeason,
}
impl ClaimType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaimType::GameStart => "GAME_START",
            ClaimType::RecentlyDropped => "RECENTLY_DROPPED",
            ClaimType::ContinuousPreseason => "CONTINUOUS_PRESEASON",
            ClaimType::ContinuousInSeason => "CONTINUOUS_IN_SEASON",
        }
    }
}
impl fmt::Display for ClaimType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CommishNote {
    pub contents_markdown: String,
    pub contents_text: String,
    #[serde(with = "serde_epoch_millis")]
    pub expiry: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentRecommendation {
    pub description_auth_safe: String,
    #[serde(rename = "last_updated_epoch_milli", with = "serde_epoch_millis")]
    pub last_updated: DateTime<Utc>,
    pub tease: String,
    pub description: String,
}

pub(crate) mod serde_epoch_millis {
    use chrono::{DateTime, Utc};
    use serde::{
        Deserialize, Deserializer, Serializer,
        de::{self, Unexpected},
    };

    // Custom serialization function for DateTime<Utc> to epoch milliseconds string
    pub fn serialize<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let epoch_millis = datetime.timestamp_millis();
        serializer.serialize_str(&epoch_millis.to_string())
    }

    // Custom deserialization function for DateTime<Utc> from epoch milliseconds string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let epoch_millis = s.as_str().parse::<i64>().map_err(|_| {
            de::Error::invalid_value(Unexpected::Str(s.as_str()), &"a string \"int64\"")
        })?;

        DateTime::<Utc>::from_timestamp_millis(epoch_millis).ok_or(de::Error::invalid_value(
            Unexpected::Str(s.as_str()),
            &"a string \"int64\" within the epoch range",
        ))
    }
}

pub(crate) mod serde_epoch_millis_opt {
    use crate::serde_epoch_millis;
    use chrono::{DateTime, Utc};
    use serde::{Deserializer, Serializer};

    // Custom serialization function for DateTime<Utc> to epoch milliseconds string
    pub fn serialize<S>(
        datetime_opt: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match datetime_opt {
            Some(datetime) => serde_epoch_millis::serialize(datetime, serializer),
            None => serializer.serialize_none(),
        }
    }

    // Custom deserialization function for DateTime<Utc> from epoch milliseconds string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Some(serde_epoch_millis::deserialize(deserializer)?))
    }
}
