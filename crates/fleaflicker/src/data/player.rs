use crate::{FormattedDecimal, Team};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeaguePlayer {
    pub pro_player: ProPlayer,
    pub requested_games: Vec<RequestedGame>,
    pub viewing_projected_points: FormattedDecimal,
    pub viewing_actual_stats: Vec<Value>,
    pub viewing_projected_stats: Vec<Value>,
    pub transaction_status: Value,
    pub requested_games_period: Value,
    pub viewing_format: String, // known values: TOTAL
    pub viewing_range: Value,
    pub owner: Option<Team>,
    pub display_group: String, // known values: RECEIVER
    pub last_x: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProPlayer {
    pub id: i32,
    pub name_full: String,
    pub name_short: String,
    pub pro_team_abbreviation: String,
    pub position: String,
    pub headshot_url: Option<String>,
    pub nfl_bye_week: i32,
    pub name_first: String,
    pub name_last: String,
    #[serde(default)]
    pub has_locked_premium_content: bool,
    pub pro_team: ProTeam,
    pub position_eligibility: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProTeam {
    pub abbreviation: String,
    pub location: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestedGame {
    pub game: Value,
    pub stats: Vec<Value>,
    pub stats_projected: Vec<Value>,
    pub participant: Option<String>, // known values: HOME
    pub period: Period,
    pub ranks: Ranks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub ordinal: i32,
    pub start_epoch_milli: String,
    pub is_now: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ranks {
    pub position: String,
    pub categories: Vec<Value>,
    pub default_points: Value,
}
