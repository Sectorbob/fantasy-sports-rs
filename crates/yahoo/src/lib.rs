#![doc = include_str!("../README.md")]

mod client;
mod data;
#[cfg(test)]
mod endpoint_tests;
pub mod filters;
#[cfg(test)]
mod oauth_tests;
pub mod token;

pub use client::{Client, Error};
pub use data::{
    Division, DraftResult, DraftStatus, DraftType, FantasyContent, FeloTier, Game, GameCode,
    GameKey, GameType, Headshot, KeeperSummary, League, LeagueKey, Manager, Matchup, OutcomeTotals,
    Pick, Player, PlayerKey, PlayerName, PlayerPool, PlayerPosition, PlayerPositions, Position,
    PositionType, Roster, RosterPosition, Scoreboard, ScoringType, Settings, Standings, Stat,
    StatModifier, Streak, Team, TeamKey, TeamLogo, TeamPoints, TeamStandings, Token,
    TradeRatifyType, Transaction, TransactionData, TransactionDataType, TransactionSourceType,
    User, WaiverType, Weeks,
};
