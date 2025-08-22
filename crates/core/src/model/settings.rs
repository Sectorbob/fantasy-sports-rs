use std::fmt;

use crate::ScoringSettings;
use chrono::{DateTime, NaiveDate, Utc};
use sleeper_fantasy_rs as sleeper;
use yahoo_fantasy_rs as yahoo;

#[derive(Debug)]
pub struct LeagueSettings {
    /// The week number in the season when the playoffs begin.
    pub has_playoffs: bool,
    pub playoff_start_week: u32,
    // pub num_of_teams: usize,
    pub num_playoff_teams: usize,
    pub divisions: Vec<String>,
    pub league_type: LeagueType,
    pub draft_type: DraftType,
    pub draft_rounds: usize,
    pub scoring: Option<ScoringSettings>,
    pub has_pick_trading: bool,
    pub waiver_type: WaiverType,
    pub max_keepers: usize,
    pub trade_deadline: TradeDeadline,
    pub start_week: usize,
    pub trade_ratify_type: TradeRatifyType,
    pub roster_spots: Vec<RosterSpot>,
}
impl LeagueSettings {
    pub fn from_yahoo(yahoo_settings: &yahoo::Settings) -> Self {
        let roster_spots = yahoo_settings
            .roster_positions
            .iter()
            .flat_map(|v| vec![RosterSpot::from(v); v.count])
            .collect::<Vec<_>>();
        let draft_rounds = roster_spots
            .iter()
            .filter(RosterSpot::is_active_spot)
            .count();

        LeagueSettings {
            has_playoffs: yahoo_settings.uses_playoff,
            playoff_start_week: yahoo_settings.playoff_start_week.map_or(0, |v| v),
            num_playoff_teams: yahoo_settings.num_playoff_teams as usize,
            divisions: yahoo_settings
                .divisions
                .iter()
                .map(|d| d.name.clone())
                .collect(),
            draft_type: DraftType::from(yahoo_settings),
            draft_rounds,
            league_type: if !yahoo_settings.uses_roster_import {
                LeagueType::Redraft
            } else {
                // TODO: how do I check if it's a keeper vs dynasty league
                LeagueType::Keeper
            },
            scoring: Some(ScoringSettings::from(yahoo_settings)),
            has_pick_trading: yahoo_settings.can_trade_draft_picks,
            waiver_type: WaiverType::from(&yahoo_settings.waiver_type),
            max_keepers: 0, // FIXME: is there a way to determine this?
            trade_deadline: TradeDeadline::from(yahoo_settings),
            start_week: yahoo_settings.start_week,
            trade_ratify_type: TradeRatifyType::from(yahoo_settings),
            roster_spots,
        }
    }
    pub fn from_sleeper(sleeper_league: &sleeper::League, draft: &Option<sleeper::Draft>) -> Self {
        let playoff_start_week = sleeper_league.settings.playoff_week_start as u32;

        let mut roster_spots = sleeper_league
            .roster_positions
            .iter()
            .map(RosterSpot::from)
            .collect::<Vec<_>>();
        let draft_rounds = roster_spots.len();
        roster_spots.extend(vec![RosterSpot::IR; sleeper_league.settings.reserve_slots]);
        LeagueSettings {
            has_playoffs: playoff_start_week > 0, //TODO: make sure this is correct logic
            playoff_start_week,
            num_playoff_teams: sleeper_league.settings.playoff_teams,
            divisions: (1..sleeper_league.settings.divisions + 1)
                .into_iter()
                .map(|id| {
                    let id = format!("division_{id}");
                    sleeper_league
                        .metadata
                        .as_ref()
                        .map_or(id.clone(), |m| m.get(id.as_str()).unwrap_or(&id).clone())
                })
                .collect(),
            draft_type: draft
                .as_ref()
                .map_or(DraftType::Snake, |d| DraftType::from(&d._type)),
            draft_rounds,
            league_type: sleeper_league.settings.r#type.into(),
            scoring: Some(ScoringSettings::from(&sleeper_league.scoring_settings)),
            has_pick_trading: sleeper_league.settings.pick_trading,
            waiver_type: WaiverType::from(sleeper_league.settings.waiver_type),
            max_keepers: sleeper_league.settings.max_keepers,
            trade_deadline: TradeDeadline::from(&sleeper_league.settings),
            start_week: sleeper_league.settings.start_week,
            trade_ratify_type: TradeRatifyType::from(&sleeper_league.settings),
            roster_spots,
        }
    }
    pub fn formatted_settings(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Type", self.league_type.to_string()),
            ("Max Keepers", self.max_keepers.to_string()),
            ("Draft Type", self.draft_type.to_string()),
            ("Draft Rounds", self.draft_rounds.to_string()),
            ("Start Week", self.start_week.to_string()),
            ("Playoffs?", self.has_playoffs.to_string()),
            ("Waiver Type", self.waiver_type.to_string()),
            ("Playoffs Start", self.playoff_start_week.to_string()),
            ("Playoff Teams", self.num_playoff_teams.to_string()),
            ("Pick Trading?", self.has_pick_trading.to_string()),
            ("Trade Approval", self.trade_ratify_type.to_string()),
            ("Trade Deadline", self.trade_deadline.to_string()),
            (
                "Roster Spots",
                self.roster_spots
                    .iter()
                    .map(RosterSpot::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            // ("", ),
        ]
    }
}

#[derive(Debug, Clone)]
pub enum LeagueType {
    Redraft,
    Keeper,
    Dynasty,
}
impl fmt::Display for LeagueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeagueType::Redraft => write!(f, "redraft"),
            LeagueType::Keeper => write!(f, "keeper"),
            LeagueType::Dynasty => write!(f, "dynasty"),
        }
    }
}
impl From<&sleeper::LeagueType> for LeagueType {
    fn from(value: &sleeper::LeagueType) -> Self {
        match value {
            sleeper::LeagueType::Redraft => LeagueType::Redraft,
            sleeper::LeagueType::Keeper => LeagueType::Keeper,
            sleeper::LeagueType::Dynasty => LeagueType::Dynasty,
        }
    }
}
impl From<sleeper::LeagueType> for LeagueType {
    fn from(value: sleeper::LeagueType) -> Self {
        LeagueType::from(&value)
    }
}

#[derive(Debug, Clone)]
pub enum DraftType {
    Snake,
    Auction,
    Linear,
}
impl fmt::Display for DraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DraftType::Snake => write!(f, "snake"),
            DraftType::Auction => write!(f, "auction"),
            DraftType::Linear => write!(f, "linear"),
        }
    }
}
impl From<&sleeper::DraftType> for DraftType {
    fn from(value: &sleeper::DraftType) -> Self {
        match value {
            sleeper::DraftType::SNAKE => DraftType::Snake,
            sleeper::DraftType::LINEAR => DraftType::Linear,
            sleeper::DraftType::AUCTION => DraftType::Auction,
        }
    }
}
impl From<sleeper::DraftType> for DraftType {
    fn from(value: sleeper::DraftType) -> Self {
        DraftType::from(&value)
    }
}
impl From<&yahoo::Settings> for DraftType {
    fn from(value: &yahoo::Settings) -> Self {
        if value.is_auction_draft {
            DraftType::Auction
        } else {
            DraftType::Snake
        }
    }
}
impl From<yahoo::Settings> for DraftType {
    fn from(value: yahoo::Settings) -> Self {
        DraftType::from(&value)
    }
}

#[derive(Debug, Clone)]
pub enum WaiverType {
    FAAB,
    ReverseStandings,
    RollingWaivers,
}
impl fmt::Display for WaiverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaiverType::FAAB => write!(f, "FAAB"),
            WaiverType::ReverseStandings => write!(f, "ReverseStandings"),
            WaiverType::RollingWaivers => write!(f, "RollingWaivers"),
        }
    }
}
impl From<&sleeper::WaiverType> for WaiverType {
    fn from(value: &sleeper::WaiverType) -> Self {
        match value {
            sleeper::WaiverType::RollingWaivers => WaiverType::RollingWaivers,
            sleeper::WaiverType::ReverseStandings => WaiverType::ReverseStandings,
            sleeper::WaiverType::FAAB => WaiverType::FAAB,
        }
    }
}
impl From<sleeper::WaiverType> for WaiverType {
    fn from(value: sleeper::WaiverType) -> Self {
        WaiverType::from(&value)
    }
}
impl From<&yahoo::WaiverType> for WaiverType {
    fn from(value: &yahoo::WaiverType) -> Self {
        match value {
            yahoo_fantasy_rs::WaiverType::FR => WaiverType::FAAB,
            yahoo_fantasy_rs::WaiverType::R => WaiverType::RollingWaivers,
        }
    }
}
impl From<yahoo::WaiverType> for WaiverType {
    fn from(value: yahoo::WaiverType) -> Self {
        WaiverType::from(&value)
    }
}

#[derive(Debug, Clone)]
pub enum TradeDeadline {
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Week(usize),
    None,
}
impl fmt::Display for TradeDeadline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeDeadline::Date(naive_date) => write!(f, "{naive_date}"),
            TradeDeadline::DateTime(date_time) => write!(f, "{date_time}"),
            TradeDeadline::Week(week) => write!(f, "Week {week}"),
            TradeDeadline::None => write!(f, "None"),
        }
    }
}
impl From<&yahoo::Settings> for TradeDeadline {
    fn from(value: &yahoo::Settings) -> Self {
        TradeDeadline::Date(value.trade_end_date)
    }
}
impl From<yahoo::Settings> for TradeDeadline {
    fn from(value: yahoo::Settings) -> Self {
        TradeDeadline::from(&value)
    }
}
impl From<&sleeper::LeagueSettings> for TradeDeadline {
    fn from(value: &sleeper::LeagueSettings) -> Self {
        TradeDeadline::Week(value.trade_deadline)
    }
}
impl From<sleeper::LeagueSettings> for TradeDeadline {
    fn from(value: sleeper::LeagueSettings) -> Self {
        TradeDeadline::from(&value)
    }
}

#[derive(Debug, Clone)]
pub enum TradeRatifyType {
    Commissioner,
    Vote,
    Other(String),
}
impl fmt::Display for TradeRatifyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeRatifyType::Commissioner => write!(f, "Commissioner"),
            TradeRatifyType::Vote => write!(f, "Vote"),
            TradeRatifyType::Other(s) => write!(f, "{s}"),
        }
    }
}
impl From<&yahoo::Settings> for TradeRatifyType {
    fn from(value: &yahoo::Settings) -> Self {
        match value.trade_ratify_type {
            yahoo::TradeRatifyType::Commish => TradeRatifyType::Commissioner,
            yahoo::TradeRatifyType::Vote => TradeRatifyType::Vote,
            yahoo::TradeRatifyType::Yahoo => TradeRatifyType::Other("Yahoo Admin".to_string()),
        }
    }
}
impl From<yahoo::Settings> for TradeRatifyType {
    fn from(value: yahoo::Settings) -> Self {
        TradeRatifyType::from(&value)
    }
}
impl From<&sleeper::LeagueSettings> for TradeRatifyType {
    fn from(_: &sleeper::LeagueSettings) -> Self {
        // For now, sleeper trade ratify type is always Commissioner
        TradeRatifyType::Commissioner
    }
}
impl From<sleeper::LeagueSettings> for TradeRatifyType {
    fn from(value: sleeper::LeagueSettings) -> Self {
        TradeRatifyType::from(&value)
    }
}

#[derive(Debug, Clone)]
pub enum RosterSpot {
    QB,
    WR,
    RB,
    TE,
    DEF,
    K,
    Flex,
    SuperFlex,
    CB,
    S,
    BN,
    IR,
}
impl RosterSpot {
    pub fn is_starting_spot(self: &&Self) -> bool {
        match self {
            RosterSpot::QB => true,
            RosterSpot::WR => true,
            RosterSpot::RB => true,
            RosterSpot::TE => true,
            RosterSpot::DEF => true,
            RosterSpot::K => true,
            RosterSpot::Flex => true,
            RosterSpot::SuperFlex => true,
            RosterSpot::CB => true,
            RosterSpot::S => true,
            RosterSpot::BN => false,
            RosterSpot::IR => false,
        }
    }
    pub fn is_active_spot(self: &&Self) -> bool {
        match self {
            RosterSpot::QB => true,
            RosterSpot::WR => true,
            RosterSpot::RB => true,
            RosterSpot::TE => true,
            RosterSpot::DEF => true,
            RosterSpot::K => true,
            RosterSpot::Flex => true,
            RosterSpot::SuperFlex => true,
            RosterSpot::CB => true,
            RosterSpot::S => true,
            RosterSpot::BN => true,
            RosterSpot::IR => false,
        }
    }
}
impl fmt::Display for RosterSpot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RosterSpot::QB => write!(f, "QB"),
            RosterSpot::WR => write!(f, "WR"),
            RosterSpot::RB => write!(f, "RB"),
            RosterSpot::TE => write!(f, "TE"),
            RosterSpot::DEF => write!(f, "DEF"),
            RosterSpot::K => write!(f, "K"),
            RosterSpot::BN => write!(f, "BN"),
            RosterSpot::IR => write!(f, "IR"),
            RosterSpot::Flex => write!(f, "Flex"),
            RosterSpot::SuperFlex => write!(f, "SuperFlex"),
            RosterSpot::CB => write!(f, "CB"),
            RosterSpot::S => write!(f, "S"),
        }
    }
}
impl From<&yahoo::RosterPosition> for RosterSpot {
    fn from(value: &yahoo::RosterPosition) -> Self {
        match &value.position {
            yahoo::Position::QB => RosterSpot::QB,
            yahoo::Position::WR => RosterSpot::WR,
            yahoo::Position::RB => RosterSpot::RB,
            yahoo::Position::TE => RosterSpot::TE,
            yahoo::Position::Flex => RosterSpot::Flex,
            yahoo::Position::K => RosterSpot::K,
            yahoo::Position::DEF => RosterSpot::DEF,
            yahoo::Position::CB => RosterSpot::CB,
            yahoo::Position::S => RosterSpot::S,
            yahoo::Position::BN => RosterSpot::BN,
            yahoo::Position::IR => RosterSpot::IR,
        }
    }
}
impl From<yahoo::RosterPosition> for RosterSpot {
    fn from(value: yahoo::RosterPosition) -> Self {
        RosterSpot::from(&value)
    }
}
impl From<&sleeper::RosterPosition> for RosterSpot {
    fn from(value: &sleeper::RosterPosition) -> Self {
        // For now, sleeper trade ratify type is always Commissioner
        match value {
            sleeper::RosterPosition::QB => RosterSpot::QB,
            sleeper::RosterPosition::RB => RosterSpot::RB,
            sleeper::RosterPosition::WR => RosterSpot::WR,
            sleeper::RosterPosition::TE => RosterSpot::TE,
            sleeper::RosterPosition::FLEX => RosterSpot::Flex,
            sleeper::RosterPosition::K => RosterSpot::K,
            sleeper::RosterPosition::DEF => RosterSpot::DEF,
            sleeper::RosterPosition::BN => RosterSpot::BN,
        }
    }
}
impl From<sleeper::RosterPosition> for RosterSpot {
    fn from(value: sleeper::RosterPosition) -> Self {
        RosterSpot::from(&value)
    }
}
