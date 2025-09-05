use crate::{FormattedDecimal, Sport, Team};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeaguePlayer {
    pub pro_player: ProPlayer,
    pub requested_games: Option<Vec<RequestedGame>>,
    pub viewing_projected_points: Option<FormattedDecimal>,
    pub viewing_actual_stats: Option<Vec<Value>>,
    pub viewing_projected_stats: Option<Vec<Value>>,
    pub transaction_status: Option<Value>,
    pub requested_games_period: Value,
    pub viewing_format: StatFormat,
    pub viewing_range: Value,
    pub owner: Option<Team>,
    pub display_group: String, // known values: RECEIVER
    pub last_x: Vec<Value>,
}
impl LeaguePlayer {
    pub fn id(&self) -> i32 {
        self.pro_player.id
    }
    pub fn name(&self) -> &str {
        &self.pro_player.name_full.as_str()
    }
    pub fn irl_team(&self) -> &str {
        &self.pro_player.pro_team_abbreviation.as_str()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProPlayer {
    pub sport: Option<Sport>,
    pub id: i32,
    pub name_full: String,
    pub name_short: String,
    pub name_first: String,
    pub name_last: String,
    pub pro_team_abbreviation: String,
    pub pro_team: ProTeam,
    pub position: String,
    pub headshot_url: Option<String>,
    pub nfl_bye_week: Option<i32>,
    pub injury: Option<Value>,
    pub news: Option<Vec<PlayerNews>>,
    #[serde(default)]
    pub is_rookie: bool,
    pub premium_content_auth_safe: Option<Value>,
    #[serde(default)]
    pub has_locked_premium_content: bool,
    pub premium_content: Option<Value>,
    pub position_eligibility: Vec<String>,
    pub external_ids: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProTeam {
    pub abbreviation: String,
    pub location: String,
    pub name: String,
    #[serde(default)]
    pub is_free_agent: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestedGame {
    pub game: Value,
    pub stats: Vec<Value>,
    pub stats_projected: Option<Vec<Value>>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerNews {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StatFormat {
    AVERAGE,
    TOTAL,
}
