//! # fantasy-sports
//!
//! One SDK for several fantasy sports platforms. Yahoo, Sleeper and Fleaflicker all expose
//! similar data (leagues, rosters, drafts, matchups, transactions) behind very different
//! APIs; this crate puts them behind a single data model and caches what it fetches on disk.
//!
//! The crate is a thin facade:
//!
//! - the platform independent model ([`League`], [`LeagueAccessor`], [`ExternalId`], ...) is
//!   re-exported from `fantasy-sports-core` at the top level, and
//! - each platform client is available as a module for when you need the raw API:
//!   [`sleeper`], [`yahoo`] and [`fleaflicker`], plus the caching HTTP [`cache`] client they
//!   are built on.
//!
//! ```no_run
//! use fantasy_sports::{ExternalId, LeagueAccessor};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mut accessor = LeagueAccessor::new().with_cache_dir("/tmp/fantasy-cache");
//! accessor.init().await?;
//!
//! // "<platform>:<id>", platform is one of sleeper, yahoo, fleaflicker
//! let id = ExternalId::try_from("sleeper:1234567890")?;
//! let league = accessor.fetch_league_context(&id, false).await?;
//! println!("{} ({})", league.name(), league.season());
//! # Ok(())
//! # }
//! ```

pub use fantasy_sports_cache as cache;
pub use fantasy_sports_fleaflicker as fleaflicker;
pub use fantasy_sports_sleeper as sleeper;
pub use fantasy_sports_yahoo as yahoo;

pub use fantasy_sports_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_ids_round_trip_for_every_platform() {
        for text in ["sleeper:123", "yahoo:359.l.1234", "fleaflicker:42"] {
            let id = ExternalId::try_from(text).expect("valid id");
            assert_eq!(text, id.to_string());
        }
    }

    #[test]
    fn unknown_platforms_are_rejected() {
        assert!(ExternalId::try_from("myspace:1").is_err());
    }

    #[test]
    fn platform_clients_are_reachable_through_the_facade() {
        let _ = sleeper::Client::new();
        let _ = fleaflicker::Client::new();
        let _ = cache::Client::new("http://localhost");
    }
}
