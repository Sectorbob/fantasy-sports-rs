# Changelog

All notable changes are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the crates follow [Semantic Versioning](https://semver.org/). All crates in the workspace share one
version number.

## [0.1.0] - Unreleased

First release. The commit history goes back to May 2025; this entry describes where the project stands.

### Added
- `fantasy-sports-cache`: `GET` client with an on-disk response cache (JSON and XML, time-to-live,
  forced refresh, bearer token per request, request counting).
- `fantasy-sports-sleeper`: client for the Sleeper API (users, leagues, rosters, drafts, matchups,
  transactions, playoff brackets, players) and, behind the `unofficial` module, its projections and stats
  endpoints.
- `fantasy-sports-yahoo`: client for the Yahoo Fantasy Sports API with OAuth2 PKCE (public client).
- `fantasy-sports-fleaflicker`: client for the Fleaflicker API.
- `fantasy-sports-core`: platform independent league model and `LeagueAccessor`, a player cache that maps
  Yahoo players onto Sleeper's, and cross-season owner records.
- `fantasy-sports`: facade crate re-exporting the above.
- `fantasy-cli` (terminal UI), not published to crates.io.

### Fixed
Bugs found while preparing this release, each now covered by a test:

*Data that was wrong or missing*
- A Sleeper league fetched without a player cache (the default for library users) came back with no
  players at all.
- 17 of Sleeper's 33 player positions (offensive and defensive line, backs, long snappers, ...) panicked
  when converted, so any roster or draft containing one crashed the load.
- Sleeper standings and rosters, Yahoo rosters and Sleeper scoreboards came back in a different order on
  every run.
- Sleeper `total_points()` read a settings key that does not exist, so it was always the "unknown" value.
- A Fleaflicker or Yahoo league with an unowned team or an unnamed manager crashed the whole load; a
  non-numeric Fleaflicker league id panicked.

*Authentication and security*
- Yahoo requests were sent without their `Authorization` header, and an expired token was never reported
  as such.
- The Yahoo token file (access and refresh tokens) was created with default, usually world-readable,
  permissions; it is now `0600`.
- Three panics on malformed responses in the Yahoo OAuth flow are errors now.

*Crashes and start-up*
- A first run on a machine without a cache directory failed to start.
- `LeagueAccessor::init` failed without Yahoo credentials, so Sleeper or Fleaflicker alone were unusable;
  Yahoo is optional now, as it is in `fantasy-cli`.
- `fantasy-cli` panicked on the matchups page of any league with an unplayed week, on small terminals,
  and when moving through an empty league list.
- `fantasy-sports-cache` did not compile on its own (it relied on another crate enabling a `serde`
  feature).

### Security
- Dependencies updated for the advisories that have fixes. The remaining ones are two unmaintained crates
  pulled in by `sled`; they are recorded with their reasons in `deny.toml`.
