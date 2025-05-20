use std::{collections::HashMap, error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
    pub news_updated: Option<i64>, // epoch millis
    pub team_changed_at: Option<i64>,

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendingPlayer {
    pub player_id: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub user_id: String,
    pub display_name: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct League {
    pub total_rosters: u8,
    pub status: LeagueStatus,
    pub sport: Sport,
    pub settings: HashMap<String, Number>,
    pub season_type: SeasonType,
    pub season: String,
    pub scoring_settings: HashMap<String, Number>,
    pub roster_positions: Vec<String>,
    pub previous_league_id: String, // "0" when there is no previous league
    pub name: String,
    pub league_id: String,
    pub draft_id: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeagueStatus {
    PreDraft,
    Drafting,
    InSeason,
    Complete,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonType {
    Regular,
    Post,
    Pre,
}

pub enum Error {
    ResourceNotFound,
    ServerError(reqwest::Error),
    DecodeError(reqwest::Error),
    ConnectError(reqwest::Error),
    UnknownError(reqwest::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("sleeper::SleeperError");

        builder.field(
            "kind",
            match self {
                Error::ResourceNotFound => &"resource not found",
                Error::ServerError(_) => &"server error",
                Error::DecodeError(_) => &"decoding error",
                Error::ConnectError(_) => &"connect error",
                Error::UnknownError(_) => &"unknown error",
            },
        );

        if let Error::ServerError(error)
        | Error::DecodeError(error)
        | Error::ConnectError(error)
        | Error::UnknownError(error) = self
        {
            if let Some(source) = error.source() {
                let message = source.to_string();
                builder.field("message", &message.as_str());
            }
        }

        builder.finish()
    }
}
