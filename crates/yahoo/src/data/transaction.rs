use core::fmt;
use std::collections::HashSet;

use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

use crate::{
    PlayerKey, TeamKey,
    data::player::{Player, Players},
};

// TODO: I want to do away with the wrapper struct (at least publicly)
#[derive(Deserialize, Debug, Clone)]
struct Transactions {
    #[serde(rename = "transaction")]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone)]
pub enum Transaction {
    Commish {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
    },
    Add {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    Drop {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    AddDrop {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    Trade {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
        picks: Vec<Pick>,
    },
    Other {
        type_name: String,
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
        rest: Vec<(String, String)>,
    },
}
impl Transaction {
    pub fn key(&self) -> String {
        match self {
            Transaction::Commish {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Add {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Drop {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Other {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::AddDrop {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Trade {
                transaction_key, ..
            } => transaction_key.clone(),
        }
    }
    pub fn timestamp(&self) -> &i64 {
        match self {
            Transaction::Commish { timestamp, .. } => timestamp,
            Transaction::Add { timestamp, .. } => timestamp,
            Transaction::Drop { timestamp, .. } => timestamp,
            Transaction::Other { timestamp, .. } => timestamp,
            Transaction::AddDrop { timestamp, .. } => timestamp,
            Transaction::Trade { timestamp, .. } => timestamp,
        }
    }

    pub fn players(&self) -> HashSet<PlayerKey> {
        match self {
            Transaction::Commish { .. } => HashSet::new(),
            Transaction::Add { players, .. } => players
                .iter()
                .flat_map(|players| players.iter().map(|p| p.player_key.clone()))
                .collect(),
            Transaction::Drop { players, .. } => players
                .iter()
                .flat_map(|players| players.iter().map(|p| p.player_key.clone()))
                .collect(),
            Transaction::AddDrop { players, .. } => players
                .iter()
                .flat_map(|players| players.iter().map(|p| p.player_key.clone()))
                .collect(),
            Transaction::Trade { players, .. } => players
                .iter()
                .flat_map(|players| players.iter().map(|p| p.player_key.clone()))
                .collect(),
            Transaction::Other { players, .. } => players
                .iter()
                .flat_map(|players| players.iter().map(|p| p.player_key.clone()))
                .collect(),
        }
    }

    pub(crate) fn deserialize_transactions<'de, D>(
        deserializer: D,
    ) -> Result<Vec<Transaction>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Transactions::deserialize(deserializer).map(|r| r.transactions)
    }
}
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Transaction struct")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Transaction, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut tx_type = None;
                let mut transaction_key = None;
                let mut transaction_id = None;
                let mut status = None;
                let mut timestamp = None;
                let mut players = None;
                let mut rest = Vec::new();
                let mut picks = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => tx_type = Some(map.next_value::<String>()?),
                        "transaction_key" => transaction_key = Some(map.next_value::<String>()?),
                        "transaction_id" => transaction_id = Some(map.next_value::<u32>()?),
                        "status" => status = Some(map.next_value::<String>()?),
                        "timestamp" => timestamp = Some(map.next_value::<i64>()?),
                        "players" => players = Some(map.next_value::<Players>()?),
                        "picks" => picks = Some(map.next_value::<Picks>()?),
                        // Collect all other fields for the "Other" variant
                        _ => {
                            let value = map.next_value::<String>()?;
                            rest.push((key, value));
                        }
                    }
                }

                let tx_type = tx_type.ok_or_else(|| de::Error::missing_field("Type"))?;
                match tx_type.as_str() {
                    "commish" => Ok(Transaction::Commish {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                    }),
                    "add" => Ok(Transaction::Add {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "drop" => Ok(Transaction::Drop {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "add/drop" => Ok(Transaction::AddDrop {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "trade" => Ok(Transaction::Trade {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                        picks: picks.map_or(vec![], |p| p.picks),
                    }),
                    other => Ok(Transaction::Other {
                        type_name: other.to_string(),
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                        rest,
                    }),
                }
            }
        }

        deserializer.deserialize_map(TransactionVisitor)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Picks {
    #[serde(rename = "pick")]
    pub picks: Vec<Pick>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pick {
    pub source_team_key: TeamKey,
    pub source_team_name: String,
    pub destination_team_key: TeamKey,
    pub destination_team_name: String,
    pub original_team_key: TeamKey,
    pub original_team_name: String,
    pub round: usize,
}
