use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    pub active: bool,
    pub player_id: String,
    pub first_name: String,
    pub last_name: String,
    pub hashtag: Option<String>,

    pub status: Option<String>, // Active, Inactive, Injured Reserve, Non Football Injury
    pub sport: Sport,
    // #[serde(default)]
    pub fantasy_positions: Option<Vec<String>>, // DL, LB, TE, DB, WR, OT, OL, OG, RB, QB, P, K, LS
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
