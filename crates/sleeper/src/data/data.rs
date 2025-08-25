use super::league::SeasonType;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Sport {
    MLB,
    NBA,
    NFL,
    OTHER(String),
}
impl std::fmt::Display for Sport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sport::MLB => write!(f, "mlb"),
            Sport::NBA => write!(f, "nba"),
            Sport::NFL => write!(f, "nfl"),
            Sport::OTHER(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Player {
    pub active: Option<bool>,
    pub player_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub hashtag: Option<String>,

    pub status: Option<String>, // Active, Inactive, Injured Reserve, Non Football Injury
    pub sport: Sport,
    // #[serde(default)]
    pub fantasy_positions: Option<Vec<String>>, // NFL: DL, LB, TE, DB, WR, OT, OL, OG, RB, QB, P, K, LS
    pub number: Option<u32>,
    pub position: Option<String>,
    pub team: Option<String>,
    pub injury_status: Option<String>,
    pub injury_body_part: Option<String>,
    pub injury_start_date: Option<String>,
    pub practice_participation: Option<String>,
    pub practice_description: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub news_updated: Option<DateTime<Utc>>, // epoch millis
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub team_changed_at: Option<DateTime<Utc>>,

    pub search_first_name: Option<String>,
    pub search_last_name: Option<String>,
    pub search_full_name: Option<String>,
    pub search_rank: Option<i32>,
    pub age: Option<u32>,
    pub height: Option<String>,
    pub weight: Option<String>,
    pub birth_date: Option<String>, // eg: "1989-12-13"
    pub birth_city: Option<String>,
    pub birth_state: Option<String>,
    pub birth_country: Option<String>,
    pub high_school: Option<String>,
    pub college: Option<String>,
    pub years_exp: Option<u32>,

    pub depth_chart_position: Option<String>,
    pub depth_chart_order: Option<u32>,

    pub stats_id: Option<u32>,
    pub rotowire_id: Option<u32>,
    pub espn_id: Option<u32>,
    pub yahoo_id: Option<u32>,
    pub fantasy_data_id: Option<u32>,
    pub rotoworld_id: Option<u32>,
    pub oddsjam_id: Option<String>,
    pub sportradar_id: Option<String>,
    pub gsis_id: Option<String>,
    pub swish_id: Option<u32>,
    pub pandascore_id: Option<String>,

    pub metadata: Option<HashMap<String, String>>,
    pub competitions: Option<Vec<String>>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.first_name {
            Some(first_name) => match &self.last_name {
                Some(last_name) => write!(f, "{} {}", first_name, last_name),
                None => write!(f, "{}", first_name),
            },
            None => match &self.last_name {
                Some(last_name) => write!(f, "{}", last_name),
                None => write!(f, "N/A"),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]

pub struct TrendingPlayer {
    pub player_id: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Roster {
    pub roster_id: u8,

    /// The unique identifier of the owner, if available. Is None when a roster
    /// is unassigned.
    pub owner_id: Option<String>,
    pub league_id: String,
    pub starters: Vec<String>,
    pub settings: HashMap<String, Number>,
    pub reserve: Option<Vec<String>>,
    pub players: Vec<String>,
    pub co_owners: Option<Vec<Value>>,
    pub keepers: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
    pub player_map: Option<Value>,
    pub taxi: Option<Value>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Matchup {
    pub points: f32,
    pub players: Vec<String>,
    pub roster_id: u8,
    pub custom_points: Option<Value>,
    pub matchup_id: Option<u8>, // can be missing when a team has a bye week or doesn't player due to playoff elimination
    pub starters: Vec<String>,
    pub starters_points: Vec<f32>,
    pub players_points: HashMap<String, f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SportState {
    /// week
    pub week: usize,
    /// pre, post, regular
    pub season_type: SeasonType,
    /// regular season start
    pub season_start_date: NaiveDate,
    /// current season
    pub season: String,
    pub previous_season: String,
    /// week of regular season
    pub leg: usize,
    /// active season for leagues
    pub league_season: String,
    /// flips in December
    pub league_create_season: String,
    /// Which week to display in UI, can be different than week
    pub display_week: usize,
    pub season_has_scores: bool,
}
