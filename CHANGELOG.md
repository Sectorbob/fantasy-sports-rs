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
- `fantasy-cli` (terminal UI) and `sectorbot` (Discord bot with an HTTP API and RSS feeds), not published
  to crates.io.

### Fixed
Bugs found while preparing this release, each now covered by a test:
- Yahoo requests were sent without their `Authorization` header, and an expired token was never reported
  as such.
- Sleeper scoreboards were returned in a different week order on every run.
- A Fleaflicker league with an unowned team failed to load, and a non-numeric Fleaflicker league id
  panicked.
- `fantasy-cli` panicked when moving through an empty league list.
- The draft order report always used one hardcoded league.
- `fantasy-sports-cache` did not compile on its own (it relied on another crate enabling a `serde`
  feature).

### Security
- Dependencies updated for the advisories that have fixes. The remaining ones are rustls 0.22 /
  rustls-webpki 0.102 pulled in by `serenity` 0.12 (sectorbot only) and two unmaintained crates pulled in by
  `sled`; they are recorded with their reasons in `deny.toml`.
