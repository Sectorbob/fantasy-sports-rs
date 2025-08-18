use core::fmt;

use serde::{
    Deserialize, Deserializer,
    de::{self, IntoDeserializer, Visitor},
};

use crate::{TransactionData, data::game::GameKey};

// TODO: I want to do away with the wrapper struct (at least publicly)
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct Players {
    #[serde(rename = "player", default)]
    pub players: Vec<Player>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]

pub struct Player {
    pub player_key: PlayerKey,
    pub player_id: u32,
    pub name: PlayerName,
    pub url: String,                      // https://sports.yahoo.com/nfl/players/7241
    pub status: Option<String>,           // NA
    pub status_full: Option<String>,      // Inactive: Coach's Decision or Not on Roster
    pub editorial_player_key: String,     // nfl.p.7241
    pub editorial_team_key: String,       // nfl.t.20
    pub editorial_team_full_name: String, // New York Jets
    pub editorial_team_abbr: String,      // NYJ
    pub editorial_team_url: String,       // https://sports.yahoo.com/nfl/teams/ny-jets/
    pub bye_weeks: Weeks,
    pub is_keeper: KeeperSummary,
    pub uniform_number: Option<String>, // can be an empty string
    pub display_position: PlayerPositions, //PlayerPosition,
    pub headshot: Headshot,
    pub image_url: String,
    pub is_undroppable: bool,
    pub position_type: PositionType,
    // pub eligible_positions:
    // pub eligible_positions_to_add:
    pub has_player_notes: Option<bool>,
    pub player_notes_last_timestamp: Option<i64>, //>1747583890</player_notes_last_timestamp>
    pub transaction_data: Option<TransactionData>,
}
impl Player {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_players<'de, D>(deserializer: D) -> Result<Vec<Player>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Players::deserialize(deserializer).map(|r| r.players)
    }
}

///
/// e.g. nfl.p.7241
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PlayerKey {
    pub game_key: GameKey, // e.g. 7241
    pub player_id: u32,
}
impl std::fmt::Display for PlayerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.p.{}", self.game_key, self.player_id)
    }
}
impl<'de> Deserialize<'de> for PlayerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_player_key(deserializer)
    }
}

fn deserialize_player_key<'de, D>(deserializer: D) -> Result<PlayerKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut parts = s.split('.');

    let game_key_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing game_key"))?
        .to_string();
    let game_key = game_key_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("game_key parse error: {}", e)))?;

    // skip the middle "l"
    let _ = parts.next();

    let player_id_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing player_id"))?;

    let player_id = player_id_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("player_id parse error: {}", e)))?;

    Ok(PlayerKey {
        game_key: GameKey::Id(game_key),
        player_id,
    })
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerName {
    pub full: String,
    pub first: String,
    pub last: String,
    pub ascii_first: String,
    pub ascii_last: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerPositions {
    One(PlayerPosition),
    Many(Vec<PlayerPosition>),
}
impl<'de> Deserialize<'de> for PlayerPositions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PlayerPositionsVisitor;

        impl<'de> Visitor<'de> for PlayerPositionsVisitor {
            type Value = PlayerPositions;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comma-delimited string of positions or a single position")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                log::debug!("Value: {}", v);
                let mut items: Vec<PlayerPosition> = v
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| PlayerPosition::deserialize(s.into_deserializer()))
                    .collect::<Result<Vec<PlayerPosition>, _>>()?;
                match items.len() {
                    0 => Err(E::custom("no positions found")),
                    1 => Ok(PlayerPositions::One(items.remove(0))),
                    _ => Ok(PlayerPositions::Many(items)),
                }
            }
        }

        deserializer.deserialize_string(PlayerPositionsVisitor)
    }
}
impl std::fmt::Display for PlayerPositions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerPositions::One(position) => write!(f, "{}", position),
            PlayerPositions::Many(positions) => {
                write!(
                    f,
                    "{}",
                    positions
                        .iter()
                        .map(|pos| pos.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum PositionType {
    K,
    DT,
    O,
    OTHER(String),
}
impl<'de> Deserialize<'de> for PositionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "K" => Ok(PositionType::K),
            "DT" => Ok(PositionType::DT),
            "O" => Ok(PositionType::O),
            other => Ok(PositionType::OTHER(other.to_string())),
        }
    }
}
impl std::fmt::Display for PositionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionType::K => write!(f, "K"),
            PositionType::DT => write!(f, "DT"),
            PositionType::O => write!(f, "O"),
            PositionType::OTHER(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlayerPosition {
    QB,
    RB,
    TE,
    WR,
    K,
    DEF,
}
impl std::fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerPosition::QB => write!(f, "QB"),
            PlayerPosition::RB => write!(f, "RB"),
            PlayerPosition::TE => write!(f, "TE"),
            PlayerPosition::WR => write!(f, "WR"),
            PlayerPosition::K => write!(f, "K"),
            PlayerPosition::DEF => write!(f, "DEF"),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Weeks {
    #[serde(rename = "week")]
    pub weeks: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct KeeperSummary {
    // #[serde(default)]
    pub status: String, // Option<bool>,
    pub cost: String,   // Option<u32>,
    // #[serde(default)]
    pub kept: String, // Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Headshot {
    pub url: String,
    pub size: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

    #[test]
    fn test_deserialize_players() {
        let xml_file_reader = BufReader::new(
            File::open("tests/data/players.xml").expect("failed to open test data file"),
        );
        let wrapper: FantasyContentWrapper =
            quick_xml::de::from_reader(xml_file_reader).expect("failed to parse test xml file");

        let players = if let FantasyContent::League(league) = wrapper.content {
            league.players
        } else {
            debug_assert!(false, "wrong type");
            return;
        };

        assert_eq!(25, players.len());
    }
}
