use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, de};

// Custom deserializer to treat empty string as None, otherwise parse as u32
pub(crate) fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        Some("") => Ok(None),
        Some(s) => T::from_str(s)
            .map(Some)
            .map_err(|e| de::Error::custom(format!("parse error: {}", e))),
        None => Ok(None),
    }
}

pub(crate) fn deserialize_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

#[derive(thiserror::Error, Debug)]
#[error("parse error: {msg}")]
pub struct ParseError {
    msg: String,
}
impl ParseError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        ParseError { msg: msg.into() }
    }
}
