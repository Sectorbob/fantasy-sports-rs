use std::{fmt, str::FromStr};

use fantasy_sports_fleaflicker as fleaflicker;
use fantasy_sports_sleeper as sleeper;
use fantasy_sports_yahoo as yahoo;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalId {
    pub platform: Platform,
    pub id: String,
}
impl ExternalId {
    pub fn new(platform: Platform, id: String) -> Self {
        ExternalId { platform, id }
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
        Err(ParseExternalIdError::from(
            "expecting a string with two segments delinted by a :",
        ))
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
        write!(f, "{}:{}", self.platform, self.id)
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
impl From<fantasy_sports_fleaflicker::League> for ExternalId {
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
    FleaFlicker,
    Sleeper,
    Yahoo,
}
impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fleaflicker" => Ok(Platform::FleaFlicker),
            "sleeper" => Ok(Platform::Sleeper),
            "yahoo" => Ok(Platform::Yahoo),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}
impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::FleaFlicker => write!(f, "fleaflicker"),
            Platform::Sleeper => write!(f, "sleeper"),
            Platform::Yahoo => write!(f, "yahoo"),
        }
    }
}
impl AsRef<[u8]> for Platform {
    fn as_ref(&self) -> &[u8] {
        match self {
            Platform::FleaFlicker => b"fleaflicker",
            Platform::Sleeper => b"sleeper",
            Platform::Yahoo => b"yahoo",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_for_every_platform() {
        for text in ["fleaflicker:2", "sleeper:3", "yahoo:359.l.1234"] {
            let id = ExternalId::try_from(text).unwrap();
            assert_eq!(text, id.to_string());
            assert_eq!(id, ExternalId::try_from(text.to_string()).unwrap());
        }
    }

    #[test]
    fn platform_names_are_case_insensitive_but_display_lowercase() {
        assert_eq!(Platform::Sleeper, "SLEEPER".parse().unwrap());
        assert_eq!(Platform::FleaFlicker, "FleaFlicker".parse().unwrap());
        assert_eq!("fleaflicker", Platform::FleaFlicker.to_string());
        assert_eq!(b"yahoo", Platform::Yahoo.as_ref());
        assert!("myspace".parse::<Platform>().is_err());
    }

    #[test]
    fn malformed_ids_are_rejected_with_a_reason() {
        for bad in ["sleeper", "", "sleeper:1:2", "myspace:1"] {
            let err = ExternalId::try_from(bad).unwrap_err();
            assert!(
                err.to_string().starts_with("invalid external id"),
                "{bad}: {err}"
            );
        }
    }

    #[test]
    fn ids_serialize_as_plain_strings() {
        let id = ExternalId::new(Platform::Sleeper, "123".into());
        assert_eq!("\"sleeper:123\"", serde_json::to_string(&id).unwrap());
        assert_eq!(id, serde_json::from_str("\"sleeper:123\"").unwrap());
        assert!(serde_json::from_str::<ExternalId>("\"nope\"").is_err());
        assert!(serde_json::from_str::<ExternalId>("123").is_err());
    }

    #[test]
    fn ids_sort_by_platform_then_id() {
        let mut ids = [
            ExternalId::new(Platform::Yahoo, "b".into()),
            ExternalId::new(Platform::Sleeper, "b".into()),
            ExternalId::new(Platform::Sleeper, "a".into()),
        ];
        ids.sort();
        assert_eq!(
            vec!["sleeper:a", "sleeper:b", "yahoo:b"],
            ids.iter().map(ExternalId::to_string).collect::<Vec<_>>()
        );
    }

    fn fixture(rel: &str) -> String {
        std::fs::read_to_string(format!("{}/../{rel}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn platform_objects_convert_into_ids() {
        let league: sleeper::League = serde_json::from_str(&fixture(
            "sleeper/testdata/v1/league/164974877262287888.json",
        ))
        .unwrap();
        let expected = ExternalId::new(Platform::Sleeper, "164974877262287888".into());
        assert_eq!(expected, ExternalId::from(&league));
        assert_eq!(expected, ExternalId::from(league));

        let users: Vec<sleeper::User> =
            serde_json::from_str(&fixture("sleeper/testdata/league_users.json")).unwrap();
        let user_id = ExternalId::from(&users[0]);
        assert_eq!(Platform::Sleeper, user_id.platform);
        assert_eq!(users[0].user_id, user_id.id);
        assert_eq!(user_id, ExternalId::from(users[0].clone()));

        let key: yahoo::LeagueKey = "359.l.564503".parse().unwrap();
        assert_eq!("yahoo:359.l.564503", ExternalId::from(&key).to_string());
        assert_eq!(ExternalId::from(&key), ExternalId::from(key));

        let player_key: yahoo::PlayerKey = "359.p.7241".parse().unwrap();
        assert_eq!(
            "yahoo:359.p.7241",
            ExternalId::from(&player_key).to_string()
        );
        assert_eq!(ExternalId::from(&player_key), ExternalId::from(player_key));
    }
}
