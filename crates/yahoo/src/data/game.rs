use core::fmt;

use serde::{Deserialize, Deserializer};

use crate::{GameCode, GameType, data::team::Teams};

#[derive(Deserialize, Debug)]
#[serde(rename = "games")]
struct Games {
    #[serde(rename = "game", default)]
    pub games: Vec<Game>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Game {
    pub game_key: u32,
    pub game_id: u32,
    pub name: String,
    pub code: GameCode,
    #[serde(rename = "type")]
    _type: GameType,
    pub url: String,
    pub season: String,
    pub is_registration_over: Option<bool>,
    pub is_game_over: Option<bool>,
    pub is_offseason: Option<bool>,
    pub teams: Option<Teams>,
}
impl Game {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_games<'de, D>(deserializer: D) -> Result<Vec<Game>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Games::deserialize(deserializer).map(|r| r.games)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameKey {
    Code(GameCode),
    Id(u32),
}
impl std::fmt::Display for GameKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameKey::Code(game_code) => write!(f, "{}", game_code.to_string()),
            GameKey::Id(id) => write!(f, "{}", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

    #[test]
    fn test_deserialize_games() {
        let reader = BufReader::new(
            File::open(
                "tests/data/fantasy/v2/games;game_codes=nfl;seasons=2009,2010,2011,2012,2013,2014,2015,2016,2017,2018,2019,2020,2021,2022,2023,2024,2025.xml",
            ).expect("failed to open test data file")
        );
        let w: FantasyContentWrapper =
            quick_xml::de::from_reader(reader).expect("failed to deserialize game context xml");
        let games = if let FantasyContent::Games(games) = w.content {
            games
        } else {
            debug_assert!(false, "fuck me");
            return;
        };

        debug_assert_eq!(17, games.len());
    }
}
