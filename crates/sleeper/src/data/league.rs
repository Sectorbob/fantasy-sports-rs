use super::Sport;
use super::helpers::{bool_from_int, int_from_bool};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ScoringSettings {
    Football {
        // Passing Stats
        pass_2pt: f32,
        pass_int: f32,
        pass_yd: f32,
        pass_td: f32,

        // Receiving Stats
        rec: f32,
        rec_2pt: f32,
        rec_td: f32,
        rec_yd: f32,

        // Rushing Stats
        rush_2pt: f32,
        rush_td: f32,
        rush_yd: f32,

        // Kicking Stats
        fgmiss: f32,
        fgm_0_19: f32,
        fgm_20_29: f32,
        fgm_30_39: f32,
        fgm_40_49: f32,
        fgm_50p: f32,
        xpm: f32,
        xpmiss: f32,

        // Other Offensive Stats
        fum: f32,
        fum_lost: f32,

        // DST Stats
        pts_allow_0: f32,
        pts_allow_1_6: f32,
        pts_allow_7_13: f32,
        pts_allow_14_20: f32,
        pts_allow_21_27: f32,
        pts_allow_28_34: f32,
        pts_allow_35p: f32,
        int: f32,
        sack: f32,
        safe: f32,
        def_td: f32,
        #[serde(default)]
        def_kr_td: f32,
        fum_rec: f32,
        #[serde(default)]
        fum_rec_td: f32,
        ff: f32,
        #[serde(default)]
        pass_int_td: f32,
        def_st_td: f32,
        def_st_fum_rec: f32,
        def_st_ff: f32,
        st_fum_rec: f32,
        st_ff: f32,
        st_td: f32,
        blk_kick: f32,
        #[serde(default)]
        kr_td: f32,
        #[serde(default)]
        def_pr_td: f32,
        #[serde(default)]
        pr_td: f32,

        // Bonus Stats
        #[serde(default)]
        bonus_rec_yd_200: f32,
        #[serde(default)]
        pass_td_40p: f32,
        #[serde(default)]
        bonus_rush_yd_200: f32,
        #[serde(default)]
        bonus_pass_yd_400: f32,
        #[serde(default)]
        rush_td_40p: f32,
        #[serde(default)]
        bonus_rec_te: f32,
        #[serde(default)]
        pass_cmp_40p: f32,
        #[serde(default)]
        rec_td_40p: f32,
        #[serde(default)]
        rush_40p: f32,
        #[serde(default)]
        rec_40p: f32,

        // Catch All Other Scoring Settings
        #[serde(flatten)]
        other: HashMap<String, Value>,
    },
    Basketball {
        // Offensive Stats
        pts: f32,
        ast: f32,
        reb: f32,
        fgmi: f32, // field goals missed
        ftmi: f32, // free throws missed
        tpm: f32,  // three pointers made?

        // Defensive Stats
        stl: f32,
        blk: f32,
        tf: f32, // TODO: turnovers forced?? or total fouls??

        // Combo Stats
        dd: f32, // double-double
        td: f32, // triple-double

        // Bad Stats
        ff: f32, // TODO: flagrant fouls??
        to: f32, // turnovers

        // Bonuses
        bonus_ast_15p: f32,
        bonus_pt_40p: f32,
        bonus_pt_50p: f32,
        bonus_reb_20p: f32,

        #[serde(flatten)]
        other: HashMap<String, Value>,
    },
}
impl ScoringSettings {
    pub fn has_other_settings(&self) -> bool {
        !self.other_settings().is_empty()
    }

    pub fn other_settings(&self) -> &HashMap<String, Value> {
        match self {
            ScoringSettings::Football { other, .. } => other,
            ScoringSettings::Basketball { other, .. } => other,
        }
    }
}

/// TODO: There is more to be gleaned from the data in GET /user/{user_id}/leagues/{sport}/{season}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct League {
    pub total_rosters: u8,
    pub status: LeagueStatus,
    pub sport: Sport,
    pub settings: LeagueSettings,
    pub season_type: SeasonType,
    pub season: String,
    pub scoring_settings: ScoringSettings,
    pub roster_positions: Vec<RosterPosition>,
    pub previous_league_id: Option<String>, // "0" or null when there is no previous league
    pub name: String,
    pub league_id: String,
    pub draft_id: String,
    pub avatar: Option<String>,
    pub last_pinned_message_id: Option<String>,
    pub last_transaction_id: Option<i64>,
    pub metadata: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub last_message_details: Option<LastMessageDetails>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}
impl League {
    pub fn has_previous_league(&self) -> bool {
        self.previous_league_id
            .as_deref()
            .is_some_and(|id| !matches!(id, "" | "0"))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeagueStatus {
    PreDraft,
    Drafting,
    InSeason,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LeagueSettings {
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub best_ball: bool,
    pub leg: Option<usize>,
    pub last_leg: Option<usize>,
    /// RESEARCH STILL NEEDED by my assumption is that this week is the final
    /// week of the playoffs. So in a typical ffl league with 13 weeks of reg
    /// season and 3 weeks of playoffs, this would be a 16.
    ///
    /// When the league is in pre-draft status, I notcied that this is None
    pub last_scored_leg: Option<usize>,
    pub max_keepers: usize,
    pub num_teams: usize,
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        // default
    )]
    pub offseason_adds: bool,
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    /// Prevent bench players from being dropped after game starts
    pub bench_locks: bool,
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub disable_trades: bool,
    #[serde(default)]
    pub divisions: usize,
    pub playoff_teams: usize,
    /// TODO: Can be 11-17, also there is a disable option
    pub playoff_week_start: usize,
    pub start_week: usize,
    pub r#type: LeagueType,
    pub waiver_type: WaiverType,
    #[serde(default)]
    pub playoff_type: PlayoffType,
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub pick_trading: bool,
    /// The final week to allow trading for your fantasy season. Trades must
    /// be accepted prior to the end of the final game of the week.
    pub trade_deadline: usize,
    #[serde(default)]
    pub reserve_slots: usize,
    #[serde(flatten)]
    pub other_data: HashMap<String, Number>,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum LeagueType {
    Redraft = 0,
    Keeper = 1,
    Dynasty = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeasonType {
    Regular,
    Post,
    Pre,
    Off, //offseason
}
impl fmt::Display for SeasonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeasonType::Regular => write!(f, "regular"),
            SeasonType::Post => write!(f, "post"),
            SeasonType::Pre => write!(f, "pre"),
            SeasonType::Off => write!(f, "off"),
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum WaiverType {
    RollingWaivers = 0,
    ReverseStandings = 1,
    FAAB = 2,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Eq, PartialEq, Debug, Default)]
#[repr(u8)]
/// Playoff Seeding Rules
pub enum PlayoffType {
    /// Teams stay on their initial side of the bracket throughout the playoffs.
    #[default]
    Default = 0,
    /// Higher seeded teams always play lower-seeded teams in every round.
    ReSeed = 1,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RosterPosition {
    QB,
    RB,
    WR,
    TE,
    FLEX,
    K,
    DEF,
    BN,
    #[serde(rename = "SUPER_FLEX")]
    SuperFlex,
    // Basketball Positions
    PG,
    SG,
    SF,
    PF,
    C,
    UTIL,
}

#[cfg(test)]
mod tests {
    use crate::LeagueSettings;

    #[test]
    fn test_league_settings_deserialization() {
        let league_settings_json = r#"{
            "last_report": 13,
            "waiver_budget": 100,
            "disable_adds": 0,
            "divisions": 3,
            "capacity_override": 0,
            "taxi_deadline": 0,
            "draft_rounds": 10,
            "reserve_allow_na": 0,
            "start_week": 3,
            "playoff_teams": 6,
            "num_teams": 12,
            "daily_waivers_hour": 0,
            "playoff_type": 0,
            "taxi_slots": 0,
            "last_scored_leg": 16,
            "daily_waivers_days": 1093,
            "playoff_week_start": 14,
            "waiver_clear_days": 2,
            "reserve_allow_doubtful": 0,
            "reserve_allow_dnr": 1,
            "taxi_allow_vets": 0,
            "waiver_day_of_week": 2,
            "reserve_allow_out": 1,
            "reserve_allow_sus": 1,
            "trade_deadline": 10,
            "taxi_years": 0,
            "daily_waivers": 0,
            "pick_trading": 1,
            "type": 1,
            "max_keepers": 3,
            "waiver_type": 2,
            "league_average_match": 0,
            "trade_review_days": 2,
            "bench_lock": 0,
            "offseason_adds": 0,
            "leg": 16,
            "reserve_slots": 1,
            "was_auto_archived": 1
        }"#;

        let league_settings: LeagueSettings = serde_json::from_str(league_settings_json)
            .expect("failed to deserialize the league settings json");
        assert_eq!(14, league_settings.playoff_week_start);
        assert_eq!(6, league_settings.playoff_teams);
    }
}
