//! Client for Sleeper's unofficial (undocumented) projections and stats endpoints.
//! These are not covered by Sleeper's public API terms and may change without notice.
mod client;
mod data;

pub use client::*;
pub use data::*;
