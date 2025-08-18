use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

use crate::data::helpers::{ParseError, deserialize_naive_date, empty_string_as_none};
use crate::{DraftResult, GameCode, GameKey, Player, Team};
use crate::{DraftStatus, Transaction};

#[derive(Deserialize, Debug)]
struct Leagues {
    #[serde(rename = "league")]
    pub leagues: Vec<League>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct League {
    pub league_key: LeagueKey,
    pub league_id: String,
    pub name: String,
    pub url: String,
    pub draft_status: DraftStatus, // known values: predraft, postdraft
    pub num_teams: u32,
    pub edit_key: u32,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub league_update_timestamp: Option<u64>,
    pub scoring_type: String,
    pub current_week: String,
    pub start_week: u32,
    pub end_week: u32,
    pub is_finished: Option<bool>,
    #[serde(default, deserialize_with = "Player::deserialize_players")]
    pub players: Vec<Player>,
    pub scoreboard: Option<Scoreboard>,
    pub settings: Option<Settings>,
    pub standings: Option<Standings>,
    #[serde(default, deserialize_with = "Team::deserialize_teams")]
    pub teams: Vec<Team>,
    #[serde(default, deserialize_with = "Transaction::deserialize_transactions")]
    pub transactions: Vec<Transaction>,
    pub league_type: String, // TODO: make this into am enum
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub start_date: NaiveDate,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub end_date: NaiveDate,
    pub season: String,
    #[serde(default, deserialize_with = "DraftResult::deserialize_draft_results")]
    pub draft_results: Vec<DraftResult>,
    pub game_code: GameCode,
    pub is_plus_league: bool,
    pub is_pro_league: bool,
    pub is_cash_league: bool,
    pub renewed: Option<Value>,
    pub renew: Option<String>, // I think this is a reference to the previous year league (if exists)
    pub weekly_deadline: Option<Value>,
}
impl League {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_leagues<'de, D>(deserializer: D) -> Result<Vec<League>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Leagues::deserialize(deserializer).map(|r| r.leagues)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeagueKey {
    game_key: GameKey,
    league_id: u32,
}
impl LeagueKey {
    pub fn new(game_key: GameKey, league_id: u32) -> Self {
        LeagueKey {
            game_key,
            league_id,
        }
    }
}
impl std::fmt::Display for LeagueKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.l.{}", self.game_key, self.league_id)
    }
}
impl FromStr for LeagueKey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        let game_key_str = parts
            .next()
            .ok_or_else(|| ParseError::new("missing game_key"))?
            .to_string();
        let game_key = game_key_str
            .parse::<u32>()
            .map_err(|e| ParseError::new(format!("game_key parse error: {}", e)))?;

        // skip the middle "l"
        let _ = parts.next();

        let league_id_str = parts
            .next()
            .ok_or_else(|| ParseError::new("missing league_id"))?;

        let league_id = league_id_str
            .parse::<u32>()
            .map_err(|e| ParseError::new(format!("league_id parse error: {}", e)))?;

        Ok(LeagueKey {
            game_key: GameKey::Id(game_key),
            league_id,
        })
    }
}
impl<'de> Deserialize<'de> for LeagueKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_league_key(deserializer)
    }
}
fn deserialize_league_key<'de, D>(deserializer: D) -> Result<LeagueKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    LeagueKey::from_str(&s)
        .map_err(|err| de::Error::custom(format!("game_key parse error: {}", err)))
}

#[derive(Deserialize, Debug, Clone)]
pub struct Scoreboard {
    pub week: u32,
    pub matchups: Matchups,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub draft_type: String,   // known values: live
    pub scoring_type: String, // known values: head
    pub uses_playoff: u8,     // 0 or 1
    pub playoff_start_week: u32,
    pub uses_playoff_reseeding: u8,     // 0 or 1
    pub uses_lock_eliminated_teams: u8, // 0 or 1
    pub uses_faab: u8,                  // 0 or 1
    pub trade_end_date: String,         // format: 2009-11-27
    pub trade_ratify_type: String,      // knwon values: commish
    pub trade_reject_time: u32,
    // roster_positions: RosterPositions,
    // stat_categories: StatCategories,
    pub divisions: Option<Divisions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Standings {
    #[serde(deserialize_with = "Team::deserialize_teams")]
    pub teams: Vec<Team>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Matchups {
    #[serde(rename = "matchup")]
    pub matchups: Vec<Matchup>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Matchup {
    pub week: Option<u32>,
    pub status: String, // known values: postevent
    pub is_tied: bool,
    pub winner_team_key: String,
    #[serde(deserialize_with = "Team::deserialize_teams")]
    pub teams: Vec<Team>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Divisions {
    #[serde(rename = "division")]
    pub divisions: Vec<Division>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Division {
    pub division_id: u32,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

    #[test]
    fn test_deserialize_leagues() {
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

    #[test]
    fn test_deserialize_divisions() {
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
