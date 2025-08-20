use crate::PositionType;
use crate::data::helpers::bool_from_int;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub draft_type: String,   // known values: live
    pub scoring_type: String, // known values: head
    pub uses_playoff: u8,     // 0 or 1
    pub playoff_start_week: u32,
    pub uses_playoff_reseeding: u8,     // 0 or 1
    pub uses_lock_eliminated_teams: u8, // 0 or 1
    pub uses_faab: u8,                  // 0 or 1
    pub trade_end_date: String,         // format: 2009-11-27
    pub trade_ratify_type: String,      // knwon values: commish
    pub trade_reject_time: u32,
    // roster_positions: RosterPositions,
    #[serde(default, deserialize_with = "Stat::deserialize_stat_categories")]
    pub stat_categories: Vec<Stat>,
    #[serde(default, deserialize_with = "StatModifier::deserialize_stat_modifiers")]
    pub stat_modifiers: Vec<StatModifier>,
    #[serde(default, deserialize_with = "Division::deserialize_divisions")]
    pub divisions: Vec<Division>,
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

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

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
    }
}
