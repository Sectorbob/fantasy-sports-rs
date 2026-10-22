use crate::{Position, SeasonType, Sport};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Projection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    pub stats: HashMap<StatKey, f64>,
    pub category: Category,
    #[serde(
        default,
        with = "chrono::serde::ts_milliseconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modified: Option<DateTime<Utc>>,
    pub week: usize,
    pub sport: Sport,
    pub season_type: SeasonType,
    #[serde(with = "i16_from_string")]
    pub season: i16,
    pub player: StatPlayerDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    pub player_id: PlayerId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    pub updated_at: Option<DateTime<Utc>>,
    pub company: StatSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent: Option<Team>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Statline {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // always None in data I've seen
    pub date: NaiveDate,
    pub stats: HashMap<StatKey, f64>,
    pub category: Category,
    #[serde(
        default,
        with = "chrono::serde::ts_milliseconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modified: Option<DateTime<Utc>>, // epoch millis most likely
    pub week: usize,
    pub sport: Sport,
    pub season_type: SeasonType,
    #[serde(with = "i16_from_string")]
    pub season: i16,
    pub player_id: PlayerId,
    pub player: StatPlayerDetails,
    pub team: Team,
    pub opponent: Team,
    pub game_id: String,
    #[serde(default, with = "chrono::serde::ts_milliseconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>, // epoch millis most likely
    pub company: StatSource, // known values: rotowire
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Proj,
    Stat,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StatPlayerDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fantasy_positions: Option<Vec<Position>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_abbr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub years_exp: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injury_body_part: Option<InjuryBodyPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injury_notes: Option<InjuryNotes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injury_start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injury_status: Option<InjuryStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(
        with = "chrono::serde::ts_milliseconds_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub news_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
pub enum InjuryStatus {
    Doubtful,
    Sus,
    DNR,
    Out,
    PUP,
    Questionable,
    NA,
    IR,
}
impl fmt::Display for InjuryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{s}",
            s = match self {
                InjuryStatus::Doubtful => "Doubtful",
                InjuryStatus::Sus => "Sus",
                InjuryStatus::DNR => "DNR",
                InjuryStatus::Out => "Out",
                InjuryStatus::PUP => "PUP",
                InjuryStatus::Questionable => "Questionable",
                InjuryStatus::NA => "NA",
                InjuryStatus::IR => "IR",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
pub enum InjuryNotes {
    Abrasion,
    Bruise,
    Dislocated,
    Fracture,
    Inflammation,
    Ruptured,
    Soreness,
    Sprain,
    Strain,
    Stinger,
    Surgery,
}
impl fmt::Display for InjuryNotes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{s}",
            s = match self {
                InjuryNotes::Bruise => "Bruise",
                InjuryNotes::Stinger => "Stinger",
                InjuryNotes::Inflammation => "Inflammation",
                InjuryNotes::Ruptured => "Ruptured",
                InjuryNotes::Fracture => "Fracture",
                InjuryNotes::Surgery => "Surgery",
                InjuryNotes::Soreness => "Soreness",
                InjuryNotes::Sprain => "Sprain",
                InjuryNotes::Dislocated => "Dislocated",
                InjuryNotes::Strain => "Strain",
                InjuryNotes::Abrasion => "Abrasion",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
pub enum InjuryBodyPart {
    Appendix,
    Arm,
    Back,
    Biceps,
    Calf,
    Foot,
    Hip,
    Neck,
    Ribs,
    Illness,
    Toe,
    Eye,
    Spine,
    Suspension,
    Oblique,
    Shoulder,
    #[serde(rename = "Knee - ACL")]
    KneeACL,
    #[serde(rename = "Knee - MCL")]
    KneeMCL,
    #[serde(rename = "Knee - PCL")]
    KneePCL,
    #[serde(rename = "Knee - ACL + MCL")]
    KneeACLMCL,
    #[serde(rename = "Knee - Meniscus")]
    KneeMeniscus,
    Pectoral,
    Heel,
    Concussion,
    Thumb,
    Head,
    Finger,
    #[serde(rename = "Lower Leg")]
    LowerLeg,
    #[serde(rename = "Upper Leg")]
    UpperLeg,
    #[serde(rename = "Lower Back")]
    LowerBack,
    Wrist,
    Shin,
    Achilles,
    Hamstring,
    Knee,
    #[serde(rename = "Shoulder - AC Joint")]
    ShoulderACJoint,
    #[serde(rename = "Shoulder - Labrum")]
    ShoulderLabrum,
    Abdomen,
    Collarbone,
    Elbow,
    Quadriceps,
    Thigh,
    Tailbone,
    Ankle,
    Forearm,
    Hand,
    Groin,
    Leg,
    #[serde(rename = "General Soreness")]
    GeneralSoreness,
    Undisclosed,
    Triceps,
    #[serde(rename = "Lower Body")]
    LowerBody,
    Personal,
    #[serde(rename = "Upper Body")]
    UpperBody,
    Chest,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
pub enum Team {
    ARI,
    ATL,
    BAL,
    BUF,
    CAR,
    CHI,
    CIN,
    CLE,
    DAL,
    DEN,
    DET,
    GB,
    HOU,
    IND,
    JAX,
    KC,
    LAC,
    LAR,
    LV,
    MIA,
    MIN,
    NE,
    NO,
    NYG,
    NYJ,
    OAK,
    PHI,
    PIT,
    SEA,
    SF,
    TB,
    TEN,
    WAS,
}
impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Team::ARI => write!(f, "ARI"),
            Team::ATL => write!(f, "ATL"),
            Team::BAL => write!(f, "BAL"),
            Team::BUF => write!(f, "BUF"),
            Team::CAR => write!(f, "CAR"),
            Team::CHI => write!(f, "CHI"),
            Team::CIN => write!(f, "CIN"),
            Team::CLE => write!(f, "CLE"),
            Team::DAL => write!(f, "DAL"),
            Team::DEN => write!(f, "DEN"),
            Team::DET => write!(f, "DET"),
            Team::GB => write!(f, "GB"),
            Team::HOU => write!(f, "HOU"),
            Team::IND => write!(f, "IND"),
            Team::JAX => write!(f, "JAX"),
            Team::KC => write!(f, "KC"),
            Team::LAC => write!(f, "LAC"),
            Team::LAR => write!(f, "LAR"),
            Team::LV => write!(f, "LV"),
            Team::MIA => write!(f, "MIA"),
            Team::MIN => write!(f, "MIN"),
            Team::NE => write!(f, "NE"),
            Team::NO => write!(f, "NO"),
            Team::NYG => write!(f, "NYG"),
            Team::NYJ => write!(f, "NYJ"),
            Team::OAK => write!(f, "OAK"),
            Team::PHI => write!(f, "PHI"),
            Team::PIT => write!(f, "PIT"),
            Team::SEA => write!(f, "SEA"),
            Team::SF => write!(f, "SF"),
            Team::TB => write!(f, "TB"),
            Team::TEN => write!(f, "TEN"),
            Team::WAS => write!(f, "WAS"),
        }
    }
}
impl FromStr for Team {
    type Err = String; // Simplified error type for this example

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("TEAM_").unwrap_or(s) {
            "ARI" => Ok(Team::ARI),
            "ATL" => Ok(Team::ATL),
            "BAL" => Ok(Team::BAL),
            "BUF" => Ok(Team::BUF),
            "CAR" => Ok(Team::CAR),
            "CHI" => Ok(Team::CHI),
            "CIN" => Ok(Team::CIN),
            "CLE" => Ok(Team::CLE),
            "DAL" => Ok(Team::DAL),
            "DEN" => Ok(Team::DEN),
            "DET" => Ok(Team::DET),
            "GB" => Ok(Team::GB),
            "HOU" => Ok(Team::HOU),
            "IND" => Ok(Team::IND),
            "JAX" => Ok(Team::JAX),
            "KC" => Ok(Team::KC),
            "LAC" => Ok(Team::LAC),
            "LAR" => Ok(Team::LAR),
            "LV" => Ok(Team::LV),
            "MIA" => Ok(Team::MIA),
            "MIN" => Ok(Team::MIN),
            "NE" => Ok(Team::NE),
            "NO" => Ok(Team::NO),
            "NYG" => Ok(Team::NYG),
            "NYJ" => Ok(Team::NYJ),
            "OAK" => Ok(Team::OAK),
            "PHI" => Ok(Team::PHI),
            "PIT" => Ok(Team::PIT),
            "SEA" => Ok(Team::SEA),
            "SF" => Ok(Team::SF),
            "TB" => Ok(Team::TB),
            "TEN" => Ok(Team::TEN),
            "WAS" => Ok(Team::WAS),
            _ => Err(format!("Invalid team abbreviation: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PlayerId {
    U32(u32),
    Team(Team),
    Other(String),
}
impl FromStr for PlayerId {
    type Err = String; // Simplified error type for this example

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<u32>() {
            Ok(PlayerId::U32(id))
        } else if let Ok(team) = s.parse::<Team>() {
            Ok(PlayerId::Team(team))
        } else {
            Ok(PlayerId::Other(s.to_string()))
        }
    }
}
impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerId::U32(id) => write!(f, "{id}"),
            PlayerId::Team(team) => write!(f, "{team}"),
            PlayerId::Other(other) => write!(f, "{other}"),
        }
    }
}
impl Serialize for PlayerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlayerId::U32(id) => serializer.serialize_u32(*id),
            PlayerId::Team(team) => serializer.serialize_str(&team.to_string()),
            PlayerId::Other(other) => serializer.serialize_str(other),
        }
    }
}
impl<'de> Deserialize<'de> for PlayerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PlayerIdVisitor; // Visitor struct to handle deserialization
        impl<'de> serde::de::Visitor<'de> for PlayerIdVisitor {
            type Value = PlayerId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a u32 or a team abbreviation string")
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PlayerId::U32(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<PlayerId>()
                    .map_err(|_| E::custom(format!("Invalid PlayerId string: {value}")))
            }
        }
        deserializer.deserialize_any(PlayerIdVisitor)
    }
}

// the variants are Sleeper's own stat key names and are (de)serialized as-is
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
pub enum StatKey {
    adp_dd_ppr,
    #[serde(rename = "blk_kick")]
    BlockedKicks,
    bonus_def_int_td_50p,
    bonus_fd_qb,
    bonus_fd_rb,
    bonus_fd_te,
    bonus_fd_wr,
    bonus_rec_rb,
    bonus_rec_te,
    bonus_rec_wr,
    bonus_tkl_10p,
    #[serde(rename = "cmp_pct")]
    CompletionPercentage,
    def_fum_td,
    def_kr_td,
    def_kr_yd,
    def_pr_td,
    def_pr_yd,
    def_snp,
    def_td,
    fga,
    fgmiss_30_39,
    fgmiss_40_49,
    fgmiss_50p,
    fgm,
    fgm_0_19,
    fgm_20_29,
    fgm_30_39,
    fgm_40_49,
    fgm_50p,
    fgm_lng,
    fgm_pct,
    fgm_yds,
    fgm_yds_over_30,
    ff,
    fum,
    fum_lost,
    fum_rec,
    gms_active,
    gp,
    gs,
    idp_def_td,
    idp_ff,
    idp_fum_rec,
    idp_fum_ret_yd,
    idp_int,
    idp_int_ret_yd,
    idp_pass_def,
    idp_pass_def_3p,
    idp_qb_hit,
    idp_safe,
    idp_sack,
    idp_sack_yd,
    idp_tkl,
    idp_tkl_ast,
    idp_tkl_loss,
    idp_tkl_solo,
    int,
    kick_pts,
    kr,
    off_snp,
    pass_2pt,
    pass_air_yd,
    pass_att,
    pass_cmp,
    pass_cmp_40p,
    pass_fd,
    pass_inc,
    pass_int,
    pass_int_td,
    pass_lng,
    pass_rtg,
    pass_rush_yd,
    pass_rz_att,
    pass_sack,
    pass_sack_yds,
    pass_td,
    pass_td_40p,
    pass_yd,
    pass_ypa,
    pass_ypc,
    penalty,
    penalty_yd,
    pos_adp_dd_ppr,
    pos_rank_half_ppr,
    pos_rank_ppr,
    pos_rank_std,
    pr,
    pr_lng,
    pr_td,
    pr_yd,
    pr_ypa,
    pts_allow,
    pts_allow_14_20,
    pts_allow_21_27,
    pts_allow_28_34,
    pts_half_ppr,
    pts_ppr,
    pts_std,
    punt_in_20,
    punt_net_yd,
    punt_tb,
    punt_yds,
    punts,
    rec,
    rec_0_4,
    rec_10_19,
    rec_2pt,
    rec_20_29,
    rec_30_39,
    rec_40p,
    rec_5_9,
    rec_air_yd,
    rec_drop,
    rec_fd,
    rec_lng,
    rec_rz_tgt,
    rec_tgt,
    rec_td,
    rec_td_40p,
    rec_td_lng,
    rec_yd,
    rec_yar,
    rec_ypr,
    rec_ypt,
    rush_2pt,
    rush_40p,
    rush_att,
    rush_fd,
    rush_lng,
    rush_rz_att,
    rush_rec_yd,
    rush_tkl_loss,
    rush_tkl_loss_yd,
    rush_td,
    rush_td_40p,
    rush_td_lng,
    rush_yd,
    rush_yac,
    rush_ypa,
    sack,
    safe,
    st_snp,
    st_td,
    st_tkl_solo,
    tkl_loss,
    tm_def_snp,
    tm_off_snp,
    tm_st_snp,
    xpa,
    xpm,
    xpmiss,
    yds_allow,
    yds_allow_300_349,
    yds_allow_350_399,
    yds_allow_400_449,
    yds_allow_550p,
    #[serde(untagged)]
    other(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum StatSource {
    Rotowire,
    SportRadar,
}

pub mod i16_from_string {
    use serde::Deserialize;

    // create a deserilize with function that will parse a i32 string to i32
    pub fn deserialize<'de, D>(deserializer: D) -> Result<i16, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<i16>().map_err(serde::de::Error::custom)
    }

    // create a serialize function that will put an i32 to a string
    pub fn serialize<S>(value: &i16, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate as core;
    use core::unofficial::data::{Projection, Statline};
    use std::collections::HashSet;

    #[test]
    fn deserialize_2023_projections() {
        let data = include_str!("../../testdata/unofficial/projections/nfl/2023/1.json");
        let projections: Vec<Projection> =
            serde_json::from_str(data).expect("unable to deserialize");
        assert!(!projections.is_empty());
        let expected_num_of_proj = 8622;
        debug_assert_eq!(8622, projections.len(), "should have 8622 total proj");

        let game_ids_for_week = projections
            .iter()
            .filter_map(|p| p.game_id.clone())
            .collect::<HashSet<String>>();
        debug_assert_eq!(
            16,
            game_ids_for_week.len(),
            "should have 16 different game_ids, week 1 should have all teams active (32 teams, 16 games)"
        );
        let mut team_ids_for_week = projections
            .iter()
            .filter_map(|p| p.team.clone())
            .collect::<HashSet<core::unofficial::Team>>()
            .into_iter()
            .collect::<Vec<_>>();
        team_ids_for_week.sort();

        debug_assert_eq!(
            33, // HUH?!?!?! I guess OAK and LV are valid team_ids in the data?
            team_ids_for_week.len(),
            "should have 32 different team_ids, week 1 should have all teams active (32 teams, 16 games)"
        );

        let statlines_where_team_present_by_opp_is_not_present: Vec<&Projection> = projections
            .iter()
            .filter(|p| p.team.is_some() && p.opponent.is_none())
            .collect();
        debug_assert_eq!(
            2160,
            statlines_where_team_present_by_opp_is_not_present.len(),
            "should have 2160 proj where team is present but opponent is not"
        );

        // let filtered: Vec<&Statline> = projections.iter().filter(|p| p.game_id.is_some()).collect();
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.game_id.is_none()).count(),
            "should have 7862 proj missing a game_id" // wtf
        );
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.date.is_none()).count(),
            "should have 7862 proj missing a date" // wtf
        );
        debug_assert_eq!(
            expected_num_of_proj,
            projections.iter().filter(|p| p.status.is_none()).count(),
            "all projections should have no status" // wtf
        );
        debug_assert_eq!(
            5702,
            projections.iter().filter(|p| p.team.is_none()).count(),
            "should have 5702 proj missing a team" // wtf
        );
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.opponent.is_none()).count(),
            "should have 7862 proj missing an opponent" // wtf
        );
        debug_assert_eq!(
            0,
            projections.iter().filter(|p| p.stats.is_empty()).count(),
            "should have no empty proj stats" // wtf
        );
        debug_assert_eq!(
            expected_num_of_proj,
            projections
                .iter()
                .filter(|p| p.stats.iter().map(|s| *s.1).sum::<f64>() > 0.0)
                .count(),
            "all proj entries have a cumluative total of 0.0 in stats values" // wtf
        );

        let statuses: HashSet<String> = projections
            .iter()
            .filter_map(|p| p.status.clone())
            .collect();
        debug_assert_eq!(HashSet::new(), statuses, "should have correct statuses");

        // serialized data formats
        debug_assert_eq!(5019072, data.len(), "original json data length");
        let reserialized_json = serde_json::to_string(&projections).unwrap();
        debug_assert_eq!(
            3533750, // improvements via optional omissions
            reserialized_json.len(),
            "reserialized json data length"
        );
    }

    #[test]
    fn deserialize_2023_statlines() {
        let data = include_bytes!("../../testdata/unofficial/stats/nfl/2023/1.json");
        let statlines: Vec<Statline> = serde_json::from_slice(data).expect("unable to deserialize");
        assert!(!statlines.is_empty());
        let expected_num_of_statlines = 2273;
        debug_assert_eq!(
            expected_num_of_statlines,
            statlines.len(),
            "should have 2273 total statlines"
        );

        let game_ids_for_week = statlines
            .iter()
            .map(|p| p.game_id.clone())
            .collect::<HashSet<String>>();
        debug_assert_eq!(
            16,
            game_ids_for_week.len(),
            "should have 16 different game_ids, week 1 should have all teams active (32 teams, 16 games)"
        );
        let mut team_ids_for_week = statlines
            .iter()
            .map(|p| p.team.clone())
            .collect::<HashSet<core::unofficial::Team>>()
            .into_iter()
            .collect::<Vec<_>>();
        team_ids_for_week.sort();
        debug_assert_eq!(
            32,
            team_ids_for_week.len(),
            "should have 32 different team_ids, week 1 should have all teams active (32 teams, 16 games)"
        );
        debug_assert_eq!(
            expected_num_of_statlines,
            statlines.iter().filter(|p| p.status.is_none()).count(),
            "all statlines should have no status" // wtf
        );
        debug_assert_eq!(
            0,
            statlines.iter().filter(|p| p.stats.is_empty()).count(),
            "should have no empty statline stats" // wtf
        );
        debug_assert_eq!(
            expected_num_of_statlines,
            statlines
                .iter()
                .filter(|p| p.stats.iter().map(|s| *s.1).sum::<f64>() > 0.0)
                .count(),
            "all proj entries have a cumluative total of 0.0 in stats values" // wtf
        );

        let statuses: HashSet<String> = statlines.iter().filter_map(|p| p.status.clone()).collect();
        debug_assert_eq!(HashSet::new(), statuses, "should have correct statuses");
    }
}
