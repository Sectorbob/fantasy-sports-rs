use std::{collections::HashMap, error::Error as StdError, fmt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number, Value};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendingPlayer {
    pub player_id: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: Option<String>, // present for get_user, but not get_users_in_league
    pub user_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    #[serde(flatten)]
    pub league_details: Option<LeagueUserDetails>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}
impl User {
    pub fn team_name(&self) -> String {
        match &self.league_details {
            Some(league_user_details) => match league_user_details.metadata.get("team_name") {
                Some(raw_team_name) => match raw_team_name.as_str() {
                    Some(team_name) => String::from(team_name),
                    None => todo!(), // data error
                },
                None => format!("Team {}", &self.display_name.as_str()),
            },
            None => format!("Team {}", &self.display_name.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_team_name() {
        use super::{LeagueUserDetails, User};
        use serde_json::json;
        use std::collections::HashMap;

        let user_1 = User {
            username: Some("bob_barker_123".to_string()),
            user_id: "3969562440090994".to_string(),
            display_name: "Bob Barker".to_string(),
            avatar: None,
            league_details: Some(LeagueUserDetails {
                is_bot: false,
                is_owner: true,
                league_id: "".to_string(),
                metadata: HashMap::from([("team_name".to_string(), json!("The Glorious One"))]),
                settings: None,
            }),
            other_data: HashMap::with_capacity(0),
        };

        assert_eq!("The Glorious One", user_1.team_name());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeagueUserDetails {
    pub is_bot: bool,
    pub is_owner: bool,
    pub league_id: String,
    pub metadata: HashMap<String, Value>,
    // #[serde(default = "empty_map")]
    pub settings: Option<HashMap<String, Value>>,
}

/// TODO: There is more to be gleaned from the data in GET /user/{user_id}/leagues/{sport}/{season}
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
    pub previous_league_id: Option<String>, // "0" or null when there is no previous league
    pub name: String,
    pub league_id: String,
    pub draft_id: String,
    pub avatar: String,
    pub last_pinned_message_id: Option<String>,
    pub last_transaction_id: Option<i64>,
    #[serde(flatten)]
    pub last_message_details: Option<LastMessageDetails>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LastMessageDetails {
    pub last_read_id: String,
    pub last_author_id: String,
    pub last_author_display_name: String,
    pub last_author_avatar: String,
    pub last_author_is_bot: bool,
    pub last_message_id: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub last_message_time: DateTime<Utc>,
    pub last_message_attachment: Option<String>,
    pub last_message_text_map: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Roster {
    pub roster_id: u8,
    pub owner_id: String,
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub status: TransactionStatus,
    #[serde(rename = "type")]
    pub _type: TransactionType,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub metadata: HashMap<String, String>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created: DateTime<Utc>,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub settings: HashMap<String, Option<Value>>,
    pub leg: u8,
    pub draft_picks: Vec<Value>,
    pub creator: String,
    pub transaction_id: String,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub adds: HashMap<String, u8>,
    pub consenter_ids: Vec<u8>,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub drops: HashMap<String, u8>,
    pub roster_ids: Vec<u8>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub status_updated: DateTime<Utc>,
    pub waiver_budget: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Waiver,
    FreeAgent,
    Trade,
}
impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Waiver => write!(f, "Waiver"),
            TransactionType::FreeAgent => write!(f, "FreeAgent"),
            TransactionType::Trade => write!(f, "Trade"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Complete,
    Failed,
    Other(String), // need
}
impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Complete => write!(f, "Complete"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Other(s) => write!(f, "{}", s),
        }
    }
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

fn null_to_empty_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: std::cmp::Eq + std::hash::Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    Option::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}
