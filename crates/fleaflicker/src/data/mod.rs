use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

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
        write!(f, "{s}", s = self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeaguesForUserResponse {
    pub leagues: Vec<League>,
    pub season: u32,
    pub can_create_new: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct League {
    pub id: i32,
    pub name: String,
    pub initials: String,
    pub sport: Option<Sport>,
    pub logo_url: String,
    pub owned_team: OwnedTeam,
    pub active_for_current_season: bool,
    pub membership_type: String, // known values: MEMBER
    pub roster_requirements: RosterRequirements,
    pub waiver_type: String, // knwon values: BLIND_BID
    pub draft_live_time_epoch_milli: String,
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
pub struct OwnedTeam {
    pub id: i32,
    pub name: String,
    pub record_overall: Record,
    pub record_division: Record,
    pub record_postseason: Record,
    pub points_for: Points,
    pub points_against: Points,
    pub streak: Points,
    pub waiver_acquisition_budget: Budget,
    pub draft_position: u8,
    pub initials: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Points {
    pub formatted: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Budget {
    pub value: f64,
    pub formatted: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub win_percentage: Points,
    pub formatted: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RosterRequirements {
    pub positions: Vec<RosterPosition>,
    pub roster_size: u8,
    pub starter_count: u8,
    pub bench_count: u8,
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
    pub colors: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeagueActivityResponse {
    pub items: Vec<LeagueActivityItem>,
    pub result_total: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeagueActivityItem {
    pub time_epoch_milli: String,
    pub membership: Option<Membership>,
    pub draft_time: Option<Value>,
    pub settings: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Membership {
    pub team: MembershipTeam,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MembershipTeam {
    pub id: i32,
    pub name: String,
    pub initials: String,
}
