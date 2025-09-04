use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;
mod boxscore;
pub use boxscore::*;
mod draftboard;
pub use draftboard::*;
mod league_activity;
pub use league_activity::*;
mod player;
pub use player::*;
mod roster;
pub use roster::*;
mod transaction;
pub use transaction::*;
mod rules;
pub use rules::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseList<T> {
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub items: Vec<T>,
    pub result_offset_next: Option<u32>,
    pub result_total: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Sport {
    NFL,
    MLB,
    NBA,
    NHL,
}
impl Sport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Sport::NFL => "NFL",
            Sport::MLB => "MLB",
            Sport::NBA => "NBA",
            Sport::NHL => "NHL",
        }
    }
}
impl fmt::Display for Sport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeaguesForUserResponse {
    pub leagues: Vec<League>,
    pub season: u32,
    #[serde(default)]
    pub can_create_new: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct League {
    pub id: i32,
    pub name: String,
    pub initials: String,
    pub sport: Option<Sport>,
    pub logo_url: Option<String>,
    pub owned_team: Team,
    pub active_for_current_season: bool,
    pub draft_status: Option<String>, // known values: POST_DRAFT
    pub membership_type: String,      // known values: MEMBER
    pub roster_requirements: RosterRequirements,
    pub waiver_type: WaiverResolutionType,
    pub draft_live_time_epoch_milli: String,
    #[serde(default)]
    pub max_keepers: u8,
    pub default_waiver_budget: u32,
    pub size: u8,
    pub capacity: u8,
    pub description: String,
    pub chat_channel: String,
    pub draft_seconds_per_selection: u32,
    // pub owners: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub sport: Option<Sport>,
    pub logo_url: Option<String>,
    pub record_overall: Option<TeamRecord>,
    pub record_division: Option<TeamRecord>,
    pub record_postseason: Option<TeamRecord>,
    pub points_for: Option<FormattedDecimal>,
    pub points_against: Option<FormattedDecimal>,
    pub streak: Option<FormattedDecimal>,
    pub waiver_position: Option<i32>,
    pub waiver_acquisition_budget: Option<FormattedDecimal>,
    pub draft_position: Option<i32>,
    pub roto: Option<TeamRotoCategories>,
    pub owners: Option<Vec<String>>,
    pub new_item_counts: Option<NewItemCounts>,
    pub initials: Option<String>,
    pub chat_channel: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormattedDecimal {
    pub value: Option<f64>,
    pub formatted: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamRecord {
    pub win: Option<i32>,
    pub losses: Option<i32>,
    pub ties: Option<i32>,
    pub win_percentage: FormattedDecimal,
    pub rank: Option<i32>,
    pub formatted: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamRotoCategories {
    pub overall_rank: i32,
    pub points: FormattedDecimal,
    pub stat_values: Vec<RotoStatValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RosterRequirements {
    pub positions: Vec<RosterPosition>,
    pub roster_size: u8,
    pub starter_count: u8,
    pub bench_count: u8,
    #[serde(default)]
    pub reserve_count: u8,
    pub eligibility: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RosterPosition {
    pub label: String,
    pub group: Option<String>,
    pub eligibility: Option<Vec<String>>,
    pub min: Option<u8>,
    pub max: Option<u8>,
    pub start: Option<u8>,
    pub colors: Option<Vec<DraftBoardCellColor>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RotoStatValue {}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewItemCounts {
    #[serde(default)]
    pub messages_unread: u32,
}
