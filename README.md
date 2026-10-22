# fantasy-sports-rs

A Rust SDK for fantasy sports platforms, and two applications built on it.

Yahoo, Sleeper and Fleaflicker all expose similar data (leagues, rosters, drafts, matchups,
transactions) through very different APIs. This workspace puts them behind one data model,
caches what it fetches on disk, and shows what that makes possible: a terminal UI for browsing
your leagues and a Discord bot that reports on them.

```text
                       ┌──────────────┐   ┌──────────────┐
                       │ fantasy-cli  │   │  sectorbot   │   applications
                       │ (terminal UI)│   │ (Discord bot)│
                       └──────┬───────┘   └──────┬───────┘
                              └─────────┬────────┘
                       ┌────────────────▼───────────────┐
                       │   fantasy-sports-core          │   one data model + league accessor
                       └───┬───────────┬────────────┬───┘
              ┌────────────▼──┐ ┌──────▼───────┐ ┌──▼────────────┐
              │ …-sleeper     │ │ …-yahoo      │ │ …-fleaflicker │   platform clients
              └────────────┬──┘ └──────┬───────┘ └──┬────────────┘
                       ┌───▼───────────▼────────────▼───┐
                       │   fantasy-sports-cache         │   GET client with an on-disk cache
                       └────────────────────────────────┘
```

The `fantasy-sports` crate is a facade that re-exports all of the above. See
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the design in more detail.

## Crates

| Crate | What it is |
|---|---|
| [`fantasy-sports`](crates/fantasy-sports) | Facade: the core model at the top level, each platform client as a module |
| [`fantasy-sports-core`](crates/core) | Platform independent leagues, rosters, drafts, matchups, standings, transactions; `LeagueAccessor` |
| [`fantasy-sports-sleeper`](crates/sleeper) | Client for the [Sleeper API](https://docs.sleeper.com/), plus its unofficial projections endpoints |
| [`fantasy-sports-yahoo`](crates/yahoo) | Client for the Yahoo Fantasy Sports API, including the OAuth2 (PKCE) flow |
| [`fantasy-sports-fleaflicker`](crates/fleaflicker) | Client for the [Fleaflicker API](https://www.fleaflicker.com/api-docs/index.html) |
| [`fantasy-sports-cache`](crates/cache) | HTTP GET client with a file system cache (JSON and XML) |
| [`fantasy-sports-cli`](apps/fantasy-cli) | `fantasy-cli`, a terminal UI over the core crate |
| [`sectorbot`](apps/sectorbot) | A Discord bot: league reports, keeper costs, RSS and feed posting |

The crates are `0.1.0` and not yet published to crates.io.

## Using the library

```rust,no_run
use fantasy_sports::{ExternalId, LeagueAccessor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut accessor = LeagueAccessor::new().with_cache_dir("/tmp/fantasy-cache");
    accessor.init().await?;

    // "<platform>:<id>" where platform is sleeper, yahoo or fleaflicker
    let id = ExternalId::try_from("sleeper:1234567890")?;
    let league = accessor.fetch_league_context(&id, false).await?;
    println!("{} ({})", league.name(), league.season());
    Ok(())
}
```

Responses are cached on disk (24 hours by default), so a second run of the same program does
not touch the network. Pass `true` as the second argument to bypass the cache.

Each platform client can also be used directly through `fantasy_sports::sleeper`,
`fantasy_sports::yahoo` and `fantasy_sports::fleaflicker`.

### Credentials

| Platform | What you need |
|---|---|
| Sleeper | Nothing, the API is public |
| Fleaflicker | Nothing, the API is public |
| Yahoo | A Yahoo developer app (client id and redirect uri). It is a *public* client using PKCE, so there is no client secret. See [crates/yahoo/notes](crates/yahoo/notes) for the auth flow scripts |

Nothing in this repository contains credentials; they are read from the environment or a config
file you create (see each app's `*.example` files).

## Building

You need a Rust toolchain that supports edition 2024 and let-chains: **Rust 1.88 or newer**.

```shell
cargo build --workspace
cargo test --workspace            # hermetic, no network or credentials needed
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo audit                       # needs cargo-audit
```

Run the apps:

```shell
# terminal UI (config lives in your OS config dir, see apps/fantasy-cli/config.example.json)
cargo run -p fantasy-sports-cli -- tui

# Discord bot (see apps/sectorbot/README.md)
cp apps/sectorbot/.env.example .env  # then fill it in
cargo run -p sectorbot
```

## Testing

The test suite is hermetic. HTTP clients are tested against [wiremock](https://crates.io/crates/wiremock)
servers that replay recorded API responses, and those recordings are **synthetic or
pseudonymized**: ids and names are consistent across files (a league still references its
rosters and users) but none of them belong to a real person.

Tests that need real credentials or the network are marked `#[ignore]`; run them yourself with
`cargo test -- --ignored` after setting the environment described in the test's doc comment.

## Status and known limitations

- The Yahoo and Fleaflicker integrations are less complete than Sleeper's.
- `sled`, used for the player cache and the bot's data store, is unmaintained; moving to a
  maintained store is on the list.
- `sectorbot` depends on `serenity` 0.12, which still uses rustls 0.22. `cargo audit` reports
  advisories for that version; they affect the Discord gateway connection only and are
  tracked until serenity updates.
- The bot's Twitter feed reads an unofficial syndication endpoint with a session cookie. It
  may break at any time and may not be permitted by Twitter's terms; it is opt-in
  (`ENABLE_FEED_FETCHER`) for that reason.
- The fixtures for the unofficial Sleeper endpoints are real, public API data.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
