mod draft;
mod game;
mod helpers;
mod league;
mod player;
mod team;
mod transaction;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::data::{
    draft::{DraftResult, DraftResults, DraftStatus},
    game::{Game, GameKey, Games},
    league::{
        Division, Divisions, League, LeagueKey, Leagues, Matchup, Scoreboard, Settings, Standings,
    },
    player::{
        Headshot, KeeperSummary, Player, PlayerKey, PlayerName, PlayerPosition, PlayerPositions,
        Players, PositionType, Weeks,
    },
    team::{
        FeloTier, Manager, OutcomeTotals, Roster, Streak, Team, TeamKey, TeamLogo, TeamPoints,
        TeamStandings, Teams,
    },
    transaction::{Transaction, Transactions},
};

#[derive(Deserialize, Debug)]
#[serde(rename = "yahoo:error")]
#[allow(dead_code)]
pub(crate) struct Error {
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "yahoo:description")]
    pub yahoo_description: Option<String>,
    pub detail: Option<String>,
}
impl Error {
    pub fn description(&self) -> String {
        self.description
            .clone()
            .or_else(|| self.yahoo_description.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}
#[derive(Deserialize, Debug)]
pub(crate) struct FantasyContentWrapper {
    #[serde(rename = "$value")]
    pub content: FantasyContent,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "$value")]
pub enum FantasyContent {
    #[serde(rename = "league")]
    League(League),
    #[serde(rename = "game")]
    Game(Game),
    #[serde(rename = "users")]
    Users(Users),
    #[serde(rename = "games")]
    Games(Games),
    #[serde(rename = "leagues")]
    Leagues(Leagues),
    #[serde(rename = "team")]
    Team(Team),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GameType {
    Full,
    PickemTeam,
    PickemGroup,
    PickemTeamList,
    Prediction,
    Squares,
}

/// Filters ths is not yet implemented. TODO
pub struct Filters {
    pub is_available: Option<bool>,
    pub game_types: Option<Vec<GameType>>,
    pub game_codes: Option<Vec<GameCode>>,
    pub seasons: Option<Vec<String>>,
}
impl Filters {
    pub fn new() -> Self {
        Filters {
            is_available: None,
            game_types: None,
            game_codes: None,
            seasons: None,
        }
    }
    pub fn set_is_available(&mut self, is_available: bool) {
        self.is_available = Some(is_available);
    }
    pub fn set_game_types(&mut self, game_types: Vec<GameType>) {
        self.game_types = Some(game_types);
    }
    pub fn set_game_codes(&mut self, game_codes: Vec<GameCode>) {
        self.game_codes = Some(game_codes);
    }
    pub fn set_seasons(&mut self, seasons: Vec<String>) {
        self.seasons = Some(seasons);
    }

    pub fn to_string(&self) -> String {
        let mut filters = Vec::new();
        // if let Some(is_available) = self.is_available {
        //     filters.push(format!("is_available={}", is_available ? "1" : "0"));
        // }
        // if let Some(game_types) = &self.game_types {
        //     for game_type in game_types {
        //         filters.push(format!("game_types={}", game_type.to_string()));
        //     }
        // }
        if let Some(game_codes) = &self.game_codes {
            for game_code in game_codes {
                filters.push(format!("game_codes={}", game_code.to_string()));
            }
        }
        if let Some(seasons) = &self.seasons {
            for season in seasons {
                filters.push(format!("seasons={}", season));
            }
        }
        filters.join(",")
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GameCode {
    NCAAF,            // NCCAF College Football Season-long
    NFL,              // NFL Season-long
    NHL,              // NHL Season-long
    NBA,              // NBA Season-long
    MLB,              // MLB Season-long
    EPLPickem,        // sogb - english premier league pickem
    BundesligaPickem, // sode - german bundesliga pickem
    LaLigaPickem,     // soes - spanish la liga pickem
    SerieAPickem,     // soit - italian Serie A pickem
    MLSPickem,        // sous - Major League Soccer Pickem
    NFLPickem,        // NFL Pickem
    NFLSurvival,      // NFL Survivor Pool
    NCAAFPickem,      // NCAAF Pickem
    PGAPickem,        // PGA Tour Pickem
    NASCARPickem,     // NASCAR Pickem
    MLBPlus,          // ran from 2002-2009
    NFLPlus,          // ran from 2001-2009
    OTHER(String),
}
impl<'de> Deserialize<'de> for GameCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "nfl" => Ok(GameCode::NFL),
            "nflp" => Ok(GameCode::NFLPickem),
            "nfls" => Ok(GameCode::NFLSurvival),
            "nhl" => Ok(GameCode::NHL),
            "nba" => Ok(GameCode::NBA),
            "mlb" => Ok(GameCode::MLB),
            "sogb" => Ok(GameCode::EPLPickem),
            "cfb" => Ok(GameCode::NCAAF),
            "ncaaf" => Ok(GameCode::NCAAFPickem),
            "pga" => Ok(GameCode::PGAPickem),
            "nascar" => Ok(GameCode::NASCARPickem),
            "pmlb" => Ok(GameCode::MLBPlus),
            "pnfl" => Ok(GameCode::NFLPlus),
            "sode" => Ok(GameCode::BundesligaPickem),
            "soes" => Ok(GameCode::LaLigaPickem),
            "soit" => Ok(GameCode::SerieAPickem),
            "sous" => Ok(GameCode::MLSPickem),
            other => Ok(GameCode::OTHER(other.to_string())),
        }
    }
}
impl fmt::Display for GameCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameCode::NFL => write!(f, "nfl"),
            GameCode::NFLPickem => write!(f, "nflp"),
            GameCode::NHL => write!(f, "nhl"),
            GameCode::NBA => write!(f, "nba"),
            GameCode::MLB => write!(f, "mlb"),
            GameCode::EPLPickem => write!(f, "sogb"),
            GameCode::BundesligaPickem => write!(f, "sode"),
            GameCode::LaLigaPickem => write!(f, "soes"),
            GameCode::SerieAPickem => write!(f, "soit"),
            GameCode::MLSPickem => write!(f, "sous"),
            GameCode::NCAAF => write!(f, "cfb"),
            GameCode::NCAAFPickem => write!(f, "ncaaf"),
            GameCode::PGAPickem => write!(f, "pga"),
            GameCode::NASCARPickem => write!(f, "nascar"),
            GameCode::NFLSurvival => write!(f, "nfls"),
            GameCode::MLBPlus => write!(f, "pmlb"),
            GameCode::NFLPlus => write!(f, "pnfl"),
            GameCode::OTHER(str) => write!(f, "{}", str),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "users")]
pub struct Users {
    #[serde(rename = "user", default)]
    pub users: Vec<User>,
}

//region User Resource
#[derive(Deserialize, Debug)]
pub struct User {
    // user_id: String,
    pub guid: String,
    pub games: Option<Games>,
    // nickname: String,
    // profile_url: String,
    // image_url: String,
    // status: String,
    // #[serde(rename = "yahoo_guid")]
    // yahoo_guid: String,
    // #[serde(rename = "yahoo_id")]
    // yahoo_id: String,
}

//endregion

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TransactionData {
    #[serde(rename = "type")]
    pub _type: String,
    pub source_type: String,             // known values: team, freeagents
    pub source_team_key: Option<String>, // will be present for a trade or drop
    pub source_team_name: Option<String>,
    pub destination_type: String,
}
