use std::{fmt, usize};

use crate::PositionType;
use crate::data::helpers::{bool_from_int, bool_from_optional_int, deserialize_naive_date};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    #[serde(default, deserialize_with = "bool_from_int")]
    pub can_trade_draft_picks: bool,
    #[serde(default)]
    pub draft_pick_time: u32,
    #[serde(deserialize_with = "bool_from_int")]
    pub draft_together: bool,
    pub draft_type: DraftType,
    #[serde(deserialize_with = "bool_from_int")]
    pub is_auction_draft: bool,
    #[serde(default, deserialize_with = "bool_from_int")]
    pub is_publicly_viewable: bool,
    #[serde(default)]
    pub num_playoff_teams: u8,
    pub player_pool: PlayerPool,
    pub playoff_start_week: Option<u32>,
    pub scoring_type: ScoringType,
    pub uses_median_score: String,
    #[serde(deserialize_with = "bool_from_int")]
    pub uses_faab: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub uses_fractional_points: bool,
    /// This is a playoffs setting. It locks the rosters for eliminaated teams
    #[serde(default, deserialize_with = "bool_from_optional_int")]
    pub uses_lock_eliminated_teams: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub uses_negative_points: bool,
    #[serde(deserialize_with = "bool_from_int")]
    pub uses_playoff: bool,
    #[serde(default, deserialize_with = "bool_from_int")]
    pub uses_playoff_reseeding: bool,
    #[serde(default, deserialize_with = "bool_from_int")]
    pub uses_roster_import: bool,
    #[serde(default = "week_start_default")]
    pub start_week: usize,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub trade_end_date: NaiveDate,
    pub trade_ratify_type: TradeRatifyType,
    pub trade_reject_time: u32,
    pub waiver_type: WaiverType,
    /// The number of days that a dropped player is on waivers before going to
    /// being a free agent
    pub waiver_time: u32,
    // roster_positions: RosterPositions,
    #[serde(default, deserialize_with = "Stat::deserialize_stat_categories")]
    pub stat_categories: Vec<Stat>,
    #[serde(default, deserialize_with = "StatModifier::deserialize_stat_modifiers")]
    pub stat_modifiers: Vec<StatModifier>,
    #[serde(default, deserialize_with = "Division::deserialize_divisions")]
    pub divisions: Vec<Division>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DraftType {
    Live, // Live Standard Draft
    #[serde(rename = "self")]
    Offline, // Offline Draft
}
impl fmt::Display for DraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DraftType::Live => write!(f, "live"),
            DraftType::Offline => write!(f, "self"),
        }
    }
}

const DEFAULT_WEEK_START: usize = 1;
fn week_start_default() -> usize {
    DEFAULT_WEEK_START
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WaiverType {
    // FAB w/ Continual rolling list tiebreak
    FR,
    // Continual rolling list
    R,
}
impl fmt::Display for WaiverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaiverType::FR => write!(f, "FR"),
            WaiverType::R => write!(f, "R"),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScoringType {
    Head,  // Head-to-Head
    Point, // Points Only
}
impl fmt::Display for ScoringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoringType::Head => write!(f, "head"),
            ScoringType::Point => write!(f, "point"),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeRatifyType {
    Commish,
    Vote,
    Yahoo,
}
impl fmt::Display for TradeRatifyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeRatifyType::Commish => write!(f, "commish"),
            TradeRatifyType::Vote => write!(f, "vote"),
            TradeRatifyType::Yahoo => write!(f, "yahoo"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct StatCategories {
    pub stats: Stats,
}

#[derive(Deserialize, Debug, Clone)]
struct Stats {
    #[serde(default, rename = "stat")]
    pub stats: Vec<Stat>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Stat {
    pub stat_id: u8,
    #[serde(deserialize_with = "bool_from_int")]
    pub enabled: bool,
    pub name: String,
    pub display_name: String,
    pub group: String,
    pub abbr: String,
    pub sort_order: i32,
    pub position_type: PositionType,
    #[serde(
        default,
        deserialize_with = "StatPositionTypes::deserialize_stat_position_types"
    )]
    pub stat_position_types: Vec<PositionType>,
}
impl Stat {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_stat_categories<'de, D>(
        deserializer: D,
    ) -> Result<Vec<Stat>, D::Error>
    where
        D: Deserializer<'de>,
    {
        StatCategories::deserialize(deserializer).map(|r| r.stats.stats)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct StatPositionTypes {
    #[serde(default, rename = "stat")]
    pub stat_position_types: Vec<PositionType>,
}
impl StatPositionTypes {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_stat_position_types<'de, D>(
        deserializer: D,
    ) -> Result<Vec<PositionType>, D::Error>
    where
        D: Deserializer<'de>,
    {
        StatPositionTypes::deserialize(deserializer).map(|r| r.stat_position_types)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct StatModifiers {
    pub stats: StatModifiersStats,
}

#[derive(Deserialize, Debug, Clone)]
struct StatModifiersStats {
    #[serde(default, rename = "stat")]
    pub stats: Vec<StatModifier>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StatModifier {
    pub stat_id: u8,
    pub value: f32,
}
impl StatModifier {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_stat_modifiers<'de, D>(
        deserializer: D,
    ) -> Result<Vec<StatModifier>, D::Error>
    where
        D: Deserializer<'de>,
    {
        StatModifiers::deserialize(deserializer).map(|r| r.stats.stats)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Divisions {
    #[serde(rename = "division")]
    pub divisions: Vec<Division>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Division {
    pub division_id: u32,
    pub name: String,
}
impl Division {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_divisions<'de, D>(deserializer: D) -> Result<Vec<Division>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Divisions::deserialize(deserializer).map(|r| r.divisions)
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlayerPool {
    All,
}
impl fmt::Display for PlayerPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerPool::All => write!(f, "ALL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{
        FantasyContent,
        data::{FantasyContentWrapper, settings::ScoringType},
    };

    #[test]
    fn test_deserialize_settings() {
        let reader = BufReader::new(
            File::open("tests/data/fantasy/v2/league/380.l.129397/settings.xml")
                .expect("failed to open test data file"),
        );
        let w: FantasyContentWrapper = quick_xml::de::from_reader(reader)
            .expect("failed to deserialize league with roster xml");
        let settings = if let FantasyContent::League(league) = w.content {
            league.settings.expect("settings not found")
        } else {
            debug_assert!(false, "fuck me");
            return;
        };

        debug_assert_eq!(3, settings.divisions.len(), "expect 3 divisions");
        debug_assert_eq!(
            37,
            settings.stat_categories.len(),
            "expect 37 stat categories"
        );
        debug_assert_eq!(
            34,
            settings.stat_modifiers.len(),
            "expect 37 stat modifiers"
        );
        debug_assert!(
            !settings.is_publicly_viewable,
            "should not be publicly viewable"
        );
        debug_assert_eq!(
            ScoringType::Head,
            settings.scoring_type,
            "should Points Only scoring type"
        );
        debug_assert!(settings.uses_playoff, "should have playoffs");
        debug_assert!(settings.uses_roster_import, "should have a roster import");
    }

    #[test]
    fn test_deserialize_settings_for_2020_other_league() {
        let reader = BufReader::new(
            File::open("tests/data/fantasy/v2/league/399.l.421906/settings.xml")
                .expect("failed to open test data file"),
        );
        let w: FantasyContentWrapper = quick_xml::de::from_reader(reader)
            .expect("failed to deserialize league with roster xml");
        let settings = if let FantasyContent::League(league) = w.content {
            league.settings.expect("settings not found")
        } else {
            debug_assert!(false, "fuck me");
            return;
        };

        debug_assert_eq!(0, settings.divisions.len(), "expect 0 divisions");
        debug_assert_eq!(
            47,
            settings.stat_categories.len(),
            "expect 37 stat categories"
        );
        debug_assert_eq!(
            45,
            settings.stat_modifiers.len(),
            "expect 37 stat modifiers"
        );
        debug_assert!(
            settings.is_publicly_viewable,
            "should not be publicly viewable"
        );
        debug_assert_eq!(
            ScoringType::Point,
            settings.scoring_type,
            "should Points Only scoring type"
        );
        debug_assert!(!settings.uses_playoff, "should not have playoffs");
        debug_assert!(
            !settings.uses_roster_import,
            "should not have a roster import"
        );
    }
}
