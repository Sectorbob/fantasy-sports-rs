use crate::{Position, SeasonType, Sport};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Statline {
    pub status: Option<String>,
    pub date: Option<NaiveDate>,
    pub stats: HashMap<String, f64>,
    pub category: String, // known values: proj
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub last_modified: Option<DateTime<Utc>>, // epoch millis most likely
    pub week: usize,
    pub sport: Sport,
    pub season_type: SeasonType,
    pub season: String,
    pub player: StatPlayerDetails,
    pub team: Option<String>,
    pub player_id: String,
    pub game_id: Option<String>,
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub updated_at: Option<DateTime<Utc>>, // epoch millis most likely
    pub company: String,          // known values: rotowire
    pub opponent: Option<String>, // eg: "NYG"
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StatPlayerDetails {
    pub fantasy_positions: Option<Vec<Position>>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub position: Option<Position>,
    pub team: Option<String>,
    pub team_abbr: Option<String>,
    pub years_exp: Option<u8>,
    pub injury_body_part: Option<String>,
    pub injury_notes: Option<String>,
    pub injury_start_date: Option<NaiveDate>,
    pub injury_status: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub news_updated: Option<DateTime<Utc>>,
}
