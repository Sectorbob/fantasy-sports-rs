use std::{fmt, str::FromStr};

use fleaflicker_fantasy_rs as fleaflicker;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
};
use sleeper_fantasy_rs as sleeper;
use yahoo_fantasy_rs as yahoo;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalId {
    pub platform: Platform,
    pub id: String,
}
impl ExternalId {
    pub fn new(platform: Platform, id: String) -> Self {
        ExternalId {
            platform: platform,
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
                    platform: Platform::from_str(parts[0]).map_err(ParseExternalIdError::from)?,

                    id: parts[1].to_string(),
                });
            } else if parts.len() > 2 {
                return Err(ParseExternalIdError::from(
                    "expecting a string with two segments delinted by a :",
                ));
            }
        }
        return Err(ParseExternalIdError::from(
            "expecting a string with two segments delinted by a :",
        ));
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
        write!(f, "{}:{}", self.platform.to_string(), self.id)
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
impl From<fleaflicker_fantasy_rs::League> for ExternalId {
    fn from(value: fleaflicker::League) -> Self {
        ExternalId {
            platform: Platform::FleaFlicker,
            id: value.id.to_string(),
        }
    }
}
impl From<&fleaflicker::League> for ExternalId {
    fn from(value: &fleaflicker::League) -> Self {
        ExternalId {
            platform: Platform::FleaFlicker,
            id: value.id.to_string(),
        }
    }
}
impl From<sleeper::League> for ExternalId {
    fn from(value: sleeper::League) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.league_id,
        }
    }
}
impl From<&sleeper::League> for ExternalId {
    fn from(value: &sleeper::League) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.league_id.clone(),
        }
    }
}
impl From<sleeper::Player> for ExternalId {
    fn from(value: sleeper::Player) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.player_id,
        }
    }
}
impl From<&sleeper::Player> for ExternalId {
    fn from(value: &sleeper::Player) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.player_id.clone(),
        }
    }
}
impl From<sleeper::User> for ExternalId {
    fn from(value: sleeper::User) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.user_id.clone(),
        }
    }
}
impl From<&sleeper::User> for ExternalId {
    fn from(value: &sleeper::User) -> Self {
        ExternalId {
            platform: Platform::Sleeper,
            id: value.user_id.clone(),
        }
    }
}
impl From<yahoo::LeagueKey> for ExternalId {
    fn from(value: yahoo::LeagueKey) -> Self {
        ExternalId {
            platform: Platform::Yahoo,
            id: value.to_string(),
        }
    }
}
impl From<&yahoo::LeagueKey> for ExternalId {
    fn from(value: &yahoo::LeagueKey) -> Self {
        ExternalId {
            platform: Platform::Yahoo,
            id: value.to_string(),
        }
    }
}
impl From<yahoo::PlayerKey> for ExternalId {
    fn from(value: yahoo::PlayerKey) -> Self {
        ExternalId {
            platform: Platform::Yahoo,
            id: value.to_string(),
        }
    }
}
impl From<&yahoo::PlayerKey> for ExternalId {
    fn from(value: &yahoo::PlayerKey) -> Self {
        ExternalId {
            platform: Platform::Yahoo,
            id: value.to_string(),
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
    FleaFlicker,
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
            "fleaflicker" => Ok(Platform::FleaFlicker),
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
            Platform::FleaFlicker => write!(f, "fleaflicker"),
            Platform::Twitter => write!(f, "twitter"),
            Platform::Youtube => write!(f, "youtube"),
            Platform::Sleeper => write!(f, "sleeper"),
            Platform::Yahoo => write!(f, "yahoo"),
        }
    }
}
impl AsRef<[u8]> for Platform {
    fn as_ref(&self) -> &[u8] {
        match self {
            Platform::Discord => b"discord",
            Platform::FleaFlicker => b"fleaflicker",
            Platform::Twitter => b"twitter",
            Platform::Youtube => b"youtube",
            Platform::Sleeper => b"sleeper",
            Platform::Yahoo => b"yahoo",
        }
    }
}
