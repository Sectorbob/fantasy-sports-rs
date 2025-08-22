mod draft;
mod game;
mod helpers;
mod league;
mod player;
mod settings;
mod team;
mod transaction;
mod user;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::data::{
    draft::{DraftResult, DraftStatus},
    game::{Game, GameKey},
    league::{League, LeagueKey, Matchup, Scoreboard, Standings},
    player::{
        Headshot, KeeperSummary, Player, PlayerKey, PlayerName, PlayerPosition, PlayerPositions,
        PositionType, Weeks,
    },
    settings::{
        Division, DraftType, PlayerPool, Position, RosterPosition, ScoringType, Settings, Stat,
        StatModifier, TradeRatifyType, WaiverType,
    },
    team::{
        FeloTier, Manager, OutcomeTotals, Roster, Streak, Team, TeamKey, TeamLogo, TeamPoints,
        TeamStandings,
    },
    transaction::{Pick, Transaction},
    user::User,
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
    #[serde(rename = "users", deserialize_with = "User::deserialize_users")]
    Users(Vec<User>),
    #[serde(rename = "games", deserialize_with = "Game::deserialize_games")]
    Games(Vec<Game>),
    #[serde(rename = "leagues", deserialize_with = "League::deserialize_leagues")]
    Leagues(Vec<League>),
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

//endregion

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TransactionData {
    #[serde(rename = "type")]
    pub r#type: TransactionDataType,
    pub source_type: TransactionSourceType, // known values: team, freeagents
    pub source_team_key: Option<TeamKey>,   // will be present for a trade or drop
    pub source_team_name: Option<String>,
    pub destination_type: TransactionSourceType,
    pub destination_team_key: Option<TeamKey>,
    pub destination_team_name: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDataType {
    Add,
    Drop,
    Trade,
}
impl fmt::Display for TransactionDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionDataType::Add => write!(f, "add"),
            TransactionDataType::Drop => write!(f, "drop"),
            TransactionDataType::Trade => write!(f, "trade"),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionSourceType {
    Team,
    Waivers,
    FreeAgents,
}
impl TransactionSourceType {
    pub fn is_team(&self) -> bool {
        match self {
            TransactionSourceType::Team => true,
            _ => false,
        }
    }
    pub fn is_waivers(&self) -> bool {
        match self {
            TransactionSourceType::Waivers => true,
            _ => false,
        }
    }
    pub fn is_free_agents(&self) -> bool {
        match self {
            TransactionSourceType::FreeAgents => true,
            _ => false,
        }
    }
}
impl fmt::Display for TransactionSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionSourceType::Team => write!(f, "team"),
            TransactionSourceType::Waivers => write!(f, "waivers"),
            TransactionSourceType::FreeAgents => write!(f, "freeagents"),
        }
    }
}
