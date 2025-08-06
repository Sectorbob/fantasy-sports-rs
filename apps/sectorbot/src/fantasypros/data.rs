use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ECRResponse {
    pub sport: String,
    pub r#type: String,
    pub ranking_type_name: String,
    pub year: String,
    pub week: String,
    pub position_id: String,
    pub scoring: String,
    pub filters: String,
    pub count: usize,
    pub total_experts: usize,
    pub last_updated: String,
    pub players: Vec<Player>,
    pub last_updated_ts: i64,
    pub experts_available: Value,
    pub accessed: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub player_id: u32,
    pub player_name: String,
    pub player_team_id: String,
    pub player_position_id: String,
    pub player_positions: String,
    pub player_short_name: String,
    pub player_eligibility: String,
    pub player_page_url: String,
    pub player_filename: String,
    pub player_bye_week: Option<String>,
    pub player_owned_avg: f32,
    pub player_ecr_delta: Option<Value>,
    pub rank_ecr: usize,
    pub rank_min: String,
    pub rank_max: String,
    pub rank_ave: String,
    pub rank_std: String,
    pub pos_rank: String,
    pub tier: u8,
}
