use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use serde_json::{Number, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]

pub struct TrendingPlayer {
    pub player_id: String,
    pub count: u32,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    /// Returns the team name of this [`User`].
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LeagueUserDetails {
    pub is_bot: bool,
    pub is_owner: bool,
    pub league_id: String,
    pub metadata: HashMap<String, Value>,
    // #[serde(default = "empty_map")]
    pub settings: Option<HashMap<String, Value>>,
}

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
        match self {
            ScoringSettings::Football {
                sack: _,
                fgm_40_49: _,
                pass_int: _,
                pts_allow_0: _,
                pass_2pt: _,
                st_td: _,
                pr_td: _,
                rec_td: _,
                fgm_30_39: _,
                kr_td: _,
                xpmiss: _,
                rush_td: _,
                def_pr_td: _,
                rec_2pt: _,
                st_fum_rec: _,
                fgmiss: _,
                ff: _,
                rec: _,
                pts_allow_14_20: _,
                fgm_0_19: _,
                def_kr_td: _,
                int: _,
                def_st_fum_rec: _,
                fum_lost: _,
                pts_allow_1_6: _,
                fgm_20_29: _,
                pts_allow_21_27: _,
                xpm: _,
                rush_2pt: _,
                fum_rec: _,
                def_st_td: _,
                fgm_50p: _,
                def_td: _,
                safe: _,
                pass_yd: _,
                blk_kick: _,
                pass_td: _,
                rush_yd: _,
                fum: _,
                pts_allow_28_34: _,
                pts_allow_35p: _,
                fum_rec_td: _,
                rec_yd: _,
                def_st_ff: _,
                pts_allow_7_13: _,
                st_ff: _,
                other,
                bonus_rec_yd_200: _,
                pass_td_40p: _,
                bonus_rush_yd_200: _,
                bonus_pass_yd_400: _,
                rush_td_40p: _,
                bonus_rec_te: _,
                pass_int_td: _,
                pass_cmp_40p: _,
                rec_td_40p: _,
                rush_40p: _,
                rec_40p: _,
            } => !other.is_empty(),
            ScoringSettings::Basketball {
                ast: _,
                blk: _,
                bonus_ast_15p: _,
                bonus_pt_40p: _,
                bonus_pt_50p: _,
                bonus_reb_20p: _,
                dd: _,
                ff: _,
                fgmi: _,
                ftmi: _,
                pts: _,
                reb: _,
                stl: _,
                td: _,
                tf: _,
                to: _,
                tpm: _,
                other,
            } => !other.is_empty(),
        }
    }

    pub fn other_settings(&self) -> &HashMap<String, Value> {
        match self {
            ScoringSettings::Football {
                sack: _,
                fgm_40_49: _,
                pass_int: _,
                pts_allow_0: _,
                pass_2pt: _,
                st_td: _,
                pr_td: _,
                rec_td: _,
                fgm_30_39: _,
                kr_td: _,
                xpmiss: _,
                rush_td: _,
                def_pr_td: _,
                rec_2pt: _,
                st_fum_rec: _,
                fgmiss: _,
                ff: _,
                rec: _,
                pts_allow_14_20: _,
                fgm_0_19: _,
                def_kr_td: _,
                int: _,
                def_st_fum_rec: _,
                fum_lost: _,
                pts_allow_1_6: _,
                fgm_20_29: _,
                pts_allow_21_27: _,
                xpm: _,
                rush_2pt: _,
                fum_rec: _,
                def_st_td: _,
                fgm_50p: _,
                def_td: _,
                safe: _,
                pass_yd: _,
                blk_kick: _,
                pass_td: _,
                rush_yd: _,
                fum: _,
                pts_allow_28_34: _,
                pts_allow_35p: _,
                fum_rec_td: _,
                rec_yd: _,
                def_st_ff: _,
                pts_allow_7_13: _,
                st_ff: _,
                other,
                bonus_rec_yd_200: _,
                pass_td_40p: _,
                bonus_rush_yd_200: _,
                bonus_pass_yd_400: _,
                rush_td_40p: _,
                bonus_rec_te: _,
                pass_int_td: _,
                pass_cmp_40p: _,
                rec_td_40p: _,
                rush_40p: _,
                rec_40p: _,
            } => other,
            ScoringSettings::Basketball {
                ast: _,
                blk: _,
                bonus_ast_15p: _,
                bonus_pt_40p: _,
                bonus_pt_50p: _,
                bonus_reb_20p: _,
                dd: _,
                ff: _,
                fgmi: _,
                ftmi: _,
                pts: _,
                reb: _,
                stl: _,
                td: _,
                tf: _,
                to: _,
                tpm: _,
                other,
            } => other,
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
    pub roster_positions: Vec<String>,
    pub previous_league_id: Option<String>, // "0" or null when there is no previous league
    pub name: String,
    pub league_id: String,
    pub draft_id: String,
    pub avatar: Option<String>,
    pub last_pinned_message_id: Option<String>,
    pub last_transaction_id: Option<i64>,
    #[serde(flatten)]
    pub last_message_details: Option<LastMessageDetails>,
    #[serde(flatten)]
    pub other_data: HashMap<String, Value>,
}
impl League {
    pub fn has_previous_league(&self) -> bool {
        match &self.previous_league_id {
            Some(previous_league_id) => match previous_league_id.as_str() {
                "" => false,
                "0" => false,
                _ => true,
            },
            None => false,
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeagueStatus {
    PreDraft,
    Drafting,
    InSeason,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeasonType {
    Regular,
    Post,
    Pre,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Roster {
    pub roster_id: u8,

    /// The unique identifier of the owner, if available. Is None when a roster
    /// is unassigned.
    pub owner_id: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Bracket {
    pub matchups: Vec<PlayoffBracketEntry>,
    pub is_losers: bool,
}
// FIXME: these don't properly determin the toilet bowl matchup
impl Bracket {
    pub fn champion(&self) -> Option<u8> {
        match self.championship_match() {
            Some(matchup_id) => match self.matchups.iter().find(|m| m.matchup_id == matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        Some(matchup.loser_roster_id)
                    } else {
                        Some(matchup.winner_roster_id)
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn runner_up(&self) -> Option<u8> {
        match self.championship_match() {
            Some(matchup_id) => match self.matchups.iter().find(|m| m.matchup_id == matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        Some(matchup.winner_roster_id)
                    } else {
                        Some(matchup.loser_roster_id)
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn championship_match(&self) -> Option<u8> {
        let last_round_matches: Option<Vec<&PlayoffBracketEntry>> =
            self.matchups.iter().max_by_key(|e| e.round).map(|e| {
                self.matchups
                    .iter()
                    .filter(|f| {
                        if f.round != e.round {
                            return false;
                        }
                        if self.is_losers {
                            if let Some(UnresolvedPlayoffSlot::WinnerOfMatch(_)) = f.team_1_from {
                                return false;
                            }
                            if let Some(UnresolvedPlayoffSlot::WinnerOfMatch(_)) = f.team_2_from {
                                return false;
                            }
                        } else {
                            // winners bracket
                            if let Some(UnresolvedPlayoffSlot::LoserOfMatch(_)) = f.team_1_from {
                                return false;
                            }
                            if let Some(UnresolvedPlayoffSlot::LoserOfMatch(_)) = f.team_2_from {
                                return false;
                            }
                        }

                        true
                    })
                    .collect()
            });
        // println!("LRM: {:#?}", last_round_matches);
        match last_round_matches {
            Some(m) => match m.len() {
                0 => None,
                1 => Some(m[0].matchup_id),
                _ => None,
            },
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PlayoffBracketEntry {
    #[serde(rename = "r")]
    pub round: u8,
    #[serde(rename = "m")]
    pub matchup_id: u8,
    #[serde(rename = "t1")]
    pub team_1_roster_id: Option<u8>, //TODO: I'd love to combine this with t1_from in an enum
    #[serde(rename = "t2")]
    pub team_2_roster_id: Option<u8>, //TODO: I'd love to combine this with t2_from in an enum
    #[serde(rename = "w")]
    pub winner_roster_id: u8,
    #[serde(rename = "l")]
    pub loser_roster_id: u8,
    #[serde(rename = "t1_from")]
    pub team_1_from: Option<UnresolvedPlayoffSlot>,
    #[serde(rename = "t2_from")]
    pub team_2_from: Option<UnresolvedPlayoffSlot>,
}
impl PlayoffBracketEntry {
    pub fn team1(&self) -> BracketTeam {
        if let Some(team_1) = self.team_1_roster_id {
            team_1.into()
        } else {
            self.team_1_from.clone().unwrap().into()
        }
    }
    pub fn team2(&self) -> BracketTeam {
        if let Some(team_2) = self.team_2_roster_id {
            team_2.into()
        } else {
            self.team_2_from.clone().unwrap().into()
        }
    }
}
impl<'de> serde::Deserialize<'de> for UnresolvedPlayoffSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OuterVisitor;

        impl<'de> Visitor<'de> for OuterVisitor {
            type Value = UnresolvedPlayoffSlot;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a nonempty sequence of a sequence of numbers")
            }

            #[inline]
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let map_value: serde_json::Map<String, serde_json::Value> =
                    serde::de::Deserialize::deserialize(
                        serde::de::value::MapAccessDeserializer::new(map),
                    )?;

                if map_value.contains_key("w") {
                    // Deserialize into VariantA
                    Ok(UnresolvedPlayoffSlot::WinnerOfMatch(
                        serde_json::from_value(map_value["w"].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))
                } else if map_value.contains_key("l") {
                    // Deserialize into VariantB
                    Ok(UnresolvedPlayoffSlot::LoserOfMatch(
                        serde_json::from_value(map_value["l"].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))
                } else {
                    // Default to VariantC
                    Err(serde::de::Error::missing_field("\"w\" or \"l\""))
                }
            }
        }
        deserializer.deserialize_map(OuterVisitor)
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum UnresolvedPlayoffSlot {
    LoserOfMatch(u8),
    WinnerOfMatch(u8),
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum BracketTeam {
    Roster(u8),
    LoserOfMatch(u8),
    WinnerOfMatch(u8),
}
impl From<u8> for BracketTeam {
    fn from(value: u8) -> Self {
        BracketTeam::Roster(value)
    }
}
impl From<UnresolvedPlayoffSlot> for BracketTeam {
    fn from(value: UnresolvedPlayoffSlot) -> Self {
        match value {
            UnresolvedPlayoffSlot::LoserOfMatch(v) => BracketTeam::LoserOfMatch(v),
            UnresolvedPlayoffSlot::WinnerOfMatch(v) => BracketTeam::WinnerOfMatch(v),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    pub draft_picks: Vec<DraftPickMove>,
    pub creator: String,
    pub transaction_id: String,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub adds: HashMap<String, u8>,
    pub consenter_ids: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "null_to_empty_map")]
    pub drops: HashMap<String, u8>,
    pub roster_ids: Vec<u8>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub status_updated: DateTime<Utc>,
    pub waiver_budget: Vec<Value>,
}
impl Transaction {
    pub fn is_free_agent(&self) -> bool {
        match self._type {
            TransactionType::FreeAgent => true,
            _ => false,
        }
    }
    pub fn is_trade(&self) -> bool {
        match self._type {
            TransactionType::Trade => true,
            _ => false,
        }
    }
    pub fn is_waiver(&self) -> bool {
        match self._type {
            TransactionType::Waiver => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Waiver,
    FreeAgent,
    Trade,
    Commissioner,
}
impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Waiver => write!(f, "Waiver"),
            TransactionType::FreeAgent => write!(f, "FreeAgent"),
            TransactionType::Trade => write!(f, "Trade"),
            TransactionType::Commissioner => write!(f, "Commissioner"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum TrendType {
    ADD,
    DROP,
}
impl fmt::Display for TrendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrendType::ADD => write!(f, "add"),
            TrendType::DROP => write!(f, "drop"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Draft {
    pub draft_id: String,
    pub league_id: String,
    pub sport: Sport,
    pub season: String,
    pub season_type: SeasonType,
    #[serde(with = "chrono::serde::ts_milliseconds", default)]
    pub created: DateTime<Utc>,
    pub creators: Option<Vec<String>>,
    pub draft_order: Option<HashMap<String, u8>>,
    pub last_message_id: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub last_message_time: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub last_picked: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
    pub settings: DraftSettings,
    /// This denotes the time the draft started. None if draft never started.
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub start_time: Option<DateTime<Utc>>,
    pub status: String,
    #[serde(rename = "type")]
    pub _type: DraftType,
    /// This will only be present when obtained directly with get_draft()
    pub slot_to_roster_id: Option<HashMap<String, u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftSettings {
    /// Sort players by name instead of ADP.
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub alpha_sort: bool,
    /// If true, a CPU will autopick for a user when a user runs out of time
    #[serde(deserialize_with = "bool_from_int", serialize_with = "int_from_bool")]
    pub cpu_autopick: bool,
    pub rounds: u8,
    pub player_type: Option<u8>,
    pub pick_timer: u32,
    pub teams: u8,
    #[serde(flatten)]
    pub other: HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DraftType {
    SNAKE,
    LINEAR,
    AUCTION,
}
impl fmt::Display for DraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DraftType::SNAKE => write!(f, "snake"),
            DraftType::LINEAR => write!(f, "linear"),
            DraftType::AUCTION => write!(f, "auction"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftPick {
    pub player_id: String,
    /// user_id this pick will go to (not all leagues have users in every slot, this can be "")
    pub picked_by: String,
    /// roster_id this pick will go to
    pub roster_id: u8,
    pub round: u8,
    /// which column this is on the draftboard
    pub draft_slot: u8,
    pub pick_no: u32,
    pub metadata: DraftPickMetadata,
    #[serde(default, deserialize_with = "null_to_false")]
    pub is_keeper: bool,
    pub draft_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftPickMove {
    pub round: usize,
    pub season: String,
    pub league_id: Option<String>,
    pub roster_id: u8,
    pub owner_id: u8,
    pub previous_owner_id: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftPickMetadata {
    pub first_name: String,
    pub injury_status: String,
    pub last_name: String,
    pub news_updated: String, // string wrapping a millis epoch
    pub number: String,
    pub player_id: String,
    pub position: String,
    pub sport: Sport,
    pub status: String, // known value: Active
    pub team: String,   // team abbreviation
    pub years_exp: String,
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i64::deserialize(deserializer)?;
    Ok(v != 0)
}

fn int_from_bool<S>(b: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *b {
        s.serialize_i8(1)
    } else {
        s.serialize_i8(0)
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

fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).map(|opt| match opt {
        Some(true) => true,
        _ => false,
    })
}

#[cfg(test)]
mod data_tests {
    use crate::{
        Draft,
        data::{BracketTeam, PlayoffBracketEntry},
    };
    use std::{fs, io};

    #[test]
    fn ensure_two_way_de_serialization_of_draft() {
        let file = fs::File::open("testdata/v1/league/164974877262287888/drafts.json")
            .expect("unable to load test data file");
        let reader = io::BufReader::new(file);

        // Deserialize it
        let drafts: Vec<Draft> =
            serde_json::from_reader(reader).expect("failed to deserialize test data file");
        println!("{:#?}", drafts);
        assert_eq!(2, drafts.len());

        // Serialize it
        let serialized_drafts =
            serde_json::to_string(&drafts).expect("failed to re-serialize the drafts json");

        let re_deserialized_drafts: Vec<Draft> = serde_json::from_str(serialized_drafts.as_str())
            .expect("failed to re-deserialize draft json object");
        assert_eq!(drafts, re_deserialized_drafts)
    }

    #[test]
    fn test_playoff_bracket_deserialization() {
        let bracket_json = r#"[{"m":1,"r":1,"l":12,"w":7,"t1":7,"t2":12},{"m":2,"r":1,"l":5,"w":9,"t1":5,"t2":9},{"m":3,"r":2,"l":4,"w":7,"t1":4,"t2":7,"t2_from":{"w":1}},{"m":4,"r":2,"l":1,"w":9,"t1":1,"t2":9,"t2_from":{"w":2}},{"p":5,"m":5,"r":2,"l":12,"w":5,"t1":12,"t2":5,"t2_from":{"l":2},"t1_from":{"l":1}},{"p":1,"m":6,"r":3,"l":9,"w":7,"t1":7,"t2":9,"t2_from":{"w":4},"t1_from":{"w":3}},{"p":3,"m":7,"r":3,"l":1,"w":4,"t1":4,"t2":1,"t2_from":{"l":4},"t1_from":{"l":3}}]"#;
        let bracket: Vec<PlayoffBracketEntry> =
            serde_json::from_str(bracket_json).expect("deserialize did not pass");
        assert_eq!(7, bracket.len());
        println!("{:#?}", bracket);
        assert_eq!(BracketTeam::Roster(7), bracket[0].team1());
        assert_eq!(BracketTeam::Roster(12), bracket[0].team2());
        assert_eq!(1, bracket[0].round);
        assert_eq!(1, bracket[0].matchup_id);

        assert_eq!(BracketTeam::Roster(5), bracket[1].team1());
        assert_eq!(BracketTeam::Roster(9), bracket[1].team2());
        assert_eq!(1, bracket[1].round);
        assert_eq!(2, bracket[1].matchup_id);

        assert_eq!(BracketTeam::Roster(4), bracket[6].team1());
        assert_eq!(BracketTeam::Roster(1), bracket[6].team2());
        assert_eq!(3, bracket[6].round);
        assert_eq!(7, bracket[6].matchup_id);
    }
}
