use super::helpers::null_to_empty_map;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt};

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
pub struct DraftPickMove {
    pub round: usize,
    pub season: String,
    pub league_id: Option<String>,
    pub roster_id: u8,
    pub owner_id: u8,
    pub previous_owner_id: u8,
}
