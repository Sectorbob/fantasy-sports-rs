# fantasy-sports-sleeper

A Rust client for the [Sleeper](https://docs.sleeper.com/) fantasy sports API. The API is public
and needs no credentials.

Covers users, leagues and their rosters/users, drafts and picks, weekly matchups, transactions,
traded picks, playoff brackets, trending players and the player universe. Responses are cached
on disk through [`fantasy-sports-cache`](https://github.com/Sectorbob/fantasy-sports-rs/tree/main/crates/cache).

```rust,no_run
use fantasy_sports_sleeper::Client;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::new();
let league = client.get_league("1234567890").await?;
println!("{} ({})", league.name, league.season);
# Ok(())
# }
```

The `unofficial` module wraps Sleeper's undocumented projections and stats endpoints. They are
not covered by Sleeper's API terms and can change without notice.

Tests replay recorded (pseudonymized) responses from `testdata/` with `wiremock`.
