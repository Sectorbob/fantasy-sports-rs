# fantasy-sports-fleaflicker

A Rust client for the [Fleaflicker API](https://www.fleaflicker.com/api-docs/index.html). The
API is public and needs no credentials.

Covers the user's leagues, league rules and standings, rosters, draft boards, league activity
and transactions, and box scores. Responses are cached on disk through
[`fantasy-sports-cache`](https://github.com/Sectorbob/fantasy-sports-rs/tree/main/crates/cache).

Tests replay recorded (pseudonymized) responses from `testdata/` with `wiremock`.
