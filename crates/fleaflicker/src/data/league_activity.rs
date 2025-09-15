use crate::{DraftSlot, LeaguePlayer, Team, serde_epoch_millis};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FutureDraftPick {
    pub season: i32,
    pub round: i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaiverResolutionTeam {
    pub team: Team,
    pub waiver_position: Option<i32>,
    pub bid: Option<i32>,
    pub invalidation_reason: Option<String>,
    pub tiebreaker_reason: Option<String>,
    #[serde(default)]
    pub loser: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum WaiverResolutionType {
    #[default]
    Priority,
    BlindBid,
    None,
}
impl WaiverResolutionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WaiverResolutionType::Priority => "PRIORITY",
            WaiverResolutionType::BlindBid => "BLIND_BID",
            WaiverResolutionType::None => "NONE",
        }
    }
}
impl fmt::Display for WaiverResolutionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeagueActivityItem {
    #[serde(rename = "timeEpochMilli", with = "serde_epoch_millis")]
    pub ts: DateTime<Utc>,
    #[serde(default, rename = "isNew")]
    pub is_new: bool,
    #[serde(flatten)]
    pub details: LeagueActivityItemDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LeagueActivityItemDetails {
    Trade {
        trade_id: i32,
        r#type: TradeType,
        team: Team,
        description: String,
    },
    #[serde(rename_all = "camelCase")]
    Settings {
        commish: Option<Team>,
        description: String,
        trade_id: Option<i32>,
    },
    #[serde(rename_all = "camelCase")]
    DraftTime {
        commish: Team,
        draft_time_epoch_milli: String,
    },
    #[serde(rename_all = "camelCase")]
    Transaction {
        r#type: Option<TransactionType>,
        player: LeaguePlayer,
        team: Team,
        #[serde(default)]
        commish_powers: bool,
        draft_pick: Option<FutureDraftPick>,
        bid_amount: Option<i32>,
        trade_id: Option<i32>,
        drafted_at_slot: Option<DraftSlot>,
        waiver_resolution_teams: Option<Vec<WaiverResolutionTeam>>,
    },
    TradeBlock {
        blerp: String,
    },
    ReserveChange {
        blargg: String,
    },
    Membership {
        team: Team,
        description: String,
    },
    CommishPowers {
        pooooooooasdf: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    TransactionAdd,
    TransactionClaim,
    TransactionDrop,
    TransactionImport,
    TransactionDraft,
    TransactionTrade,
}
impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::TransactionAdd => "TRANSACTION_ADD",
            TransactionType::TransactionClaim => "TRANSACTION_CLAIM",
            TransactionType::TransactionDrop => "TRANSACTION_DROP",
            TransactionType::TransactionImport => "TRANSACTION_IMPORT",
            TransactionType::TransactionDraft => "TRANSACTION_DRAFT",
            TransactionType::TransactionTrade => "TRANSACTION_TRADE",
        }
    }
}
impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Default)]
pub enum TradeType {
    #[default]
    TradeAccepted,
    TradeAcceptedFinal,
    TradeCancelled,
    TradeExecutionDelayed,
    TradeInvalidated,
    TradeMessage,
    TradeProposed,
    TradeRejected,
    TradeRejectedMessage,
    TradeToReview,
    TradeVetoed,
}
impl TradeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TradeType::TradeAccepted => "TRADE_ACCEPTED",
            TradeType::TradeAcceptedFinal => "TRADE_ACCEPTED_FINAL",
            TradeType::TradeCancelled => "TRADE_CANCELLED",
            TradeType::TradeExecutionDelayed => "TRADE_EXECUTION_DELAYED",
            TradeType::TradeInvalidated => "TRADE_INVALIDATED",
            TradeType::TradeMessage => "TRADE_MESSAGE",
            TradeType::TradeProposed => "TRADE_PROPOSED",
            TradeType::TradeRejected => "TRADE_REJECTED",
            TradeType::TradeRejectedMessage => "TRADE_REJECTED_MESSAGE",
            TradeType::TradeToReview => "TRADE_TO_REVIEW",
            TradeType::TradeVetoed => "TRADE_VETOED",
        }
    }
}
impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
