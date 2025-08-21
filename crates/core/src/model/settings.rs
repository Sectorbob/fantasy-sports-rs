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
    pub num_playoff_teams: usize,
    pub divisions: Vec<String>,
    pub league_type: LeagueType,
    pub draft_type: DraftType,
    pub scoring: Option<ScoringSettings>,
    pub has_pick_trading: bool,
    pub waiver_type: WaiverType,
    pub max_keepers: usize,
    pub trade_deadline: TradeDeadline,
    pub start_week: usize,
    pub trade_ratify_type: TradeRatifyType,
}
impl LeagueSettings {
    pub fn from_yahoo(yahoo_settings: &yahoo::Settings) -> Self {
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
            league_type: if !yahoo_settings.uses_roster_import {
                LeagueType::Redraft
            } else {
                // how do I check if it's a keeper vs dynasty league
                todo!()
            },
            scoring: Some(ScoringSettings::from(yahoo_settings)),
            has_pick_trading: yahoo_settings.can_trade_draft_picks,
            waiver_type: WaiverType::from(&yahoo_settings.waiver_type),
            max_keepers: 0, // FIXME: is there a way to determine this?
            trade_deadline: TradeDeadline::from(yahoo_settings),
            start_week: yahoo_settings.start_week,
            trade_ratify_type: TradeRatifyType::from(yahoo_settings),
        }
    }
    pub fn from_sleeper(sleeper_league: &sleeper::League, draft: &Option<sleeper::Draft>) -> Self {
        let playoff_start_week = sleeper_league.settings.playoff_week_start as u32;
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
            league_type: sleeper_league.settings.r#type.into(),
            scoring: Some(ScoringSettings::from(&sleeper_league.scoring_settings)),
            has_pick_trading: sleeper_league.settings.pick_trading,
            waiver_type: WaiverType::from(sleeper_league.settings.waiver_type),
            max_keepers: sleeper_league.settings.max_keepers,
            trade_deadline: TradeDeadline::from(&sleeper_league.settings),
            start_week: sleeper_league.settings.start_week,
            trade_ratify_type: TradeRatifyType::from(&sleeper_league.settings),
        }
    }
    pub fn formatted_settings(&self) -> Vec<(&'static str, String)> {
        vec![
            ("league_type", self.league_type.to_string()),
            ("draft_type", self.draft_type.to_string()),
            ("has_playoffs", self.has_playoffs.to_string()),
            ("waiver_type", self.waiver_type.to_string()),
            ("playoff_start_week", self.playoff_start_week.to_string()),
            ("num_playoff_teams", self.num_playoff_teams.to_string()),
            ("max_keepers", self.max_keepers.to_string()),
            ("has_pick_trading", self.has_pick_trading.to_string()),
            ("trade_ratify_type", self.trade_ratify_type.to_string()),
            ("trade_deadline", self.trade_deadline.to_string()),
            ("start_week", self.start_week.to_string()),
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
