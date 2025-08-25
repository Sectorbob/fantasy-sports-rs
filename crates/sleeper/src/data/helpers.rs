use serde::{Deserialize, Deserializer, Serializer};
use std::collections::HashMap;

pub(crate) fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = i64::deserialize(deserializer)?;
    Ok(v != 0)
}

pub(crate) fn int_from_bool<S>(b: &bool, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *b {
        s.serialize_i8(1)
    } else {
        s.serialize_i8(0)
    }
}

pub(crate) fn null_to_empty_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: std::cmp::Eq + std::hash::Hash + Deserialize<'de>,
    V: Deserialize<'de>,
{
    Option::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

pub(crate) fn null_to_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).map(|opt| match opt {
        Some(true) => true,
        _ => false,
    })
}
