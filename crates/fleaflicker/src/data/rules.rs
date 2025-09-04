use crate::{FormattedDecimal, RosterPosition};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeagueRules {
    pub roster_positions: Vec<RosterPosition>,
    pub groups: Vec<ScoringRuleGroup>,
    pub num_starters: u32,
    pub num_bench: u32,
    pub max_active: u32,
    pub max_roster_size: u32,
    pub all_scoring_rule_types: Vec<ScoringRuleType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoringRuleGroup {
    pub label: String,
    pub scoring_rules: Vec<ScoringRule>,
    pub all_categories: Vec<StatCategory>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoringRule {
    pub category: StatCategory,
    pub points: FormattedDecimal,
    pub for_every: Option<i32>,
    pub bound_lower: Option<i32>,
    pub description: String,
    pub apply_to: Option<Vec<String>>,
    #[serde(default)]
    pub is_bonus: bool,
    pub template: String,
    pub range_type: Option<String>, // known values: RANGE_LOWER_BOUND, RANGE_DOUBLE_BOUND
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScoringRuleType {
    pub template: String,
    #[serde(default)]
    pub has_lower_bound: bool,
    #[serde(default)]
    pub has_upper_bound: bool,
    #[serde(default)]
    pub is_multi_value: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatCategory {
    pub id: i32,
    pub abbreviation: String,
    pub name_singular: String,
    pub name_plural: String,
    pub summary_label_if_differs_from_abbreviation: Option<String>,
    #[serde(default)]
    pub lower_is_better: bool,
}
