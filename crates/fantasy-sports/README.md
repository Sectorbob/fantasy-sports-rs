# fantasy-sports

One Rust SDK for several fantasy sports platforms. Yahoo, Sleeper and Fleaflicker expose similar
data through very different APIs; this crate puts them behind a single data model and caches
what it fetches on disk.

This crate is a facade. The platform independent model is re-exported at the top level, and each
platform client is a module (`sleeper`, `yahoo`, `fleaflicker`) plus the caching HTTP client
(`cache`) they share.

```rust,no_run
use fantasy_sports::{ExternalId, LeagueAccessor};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let mut accessor = LeagueAccessor::new().with_cache_dir("/tmp/fantasy-cache");
accessor.init().await?;

let id = ExternalId::try_from("sleeper:1234567890")?;
let league = accessor.fetch_league_context(&id, false).await?;
println!("{} ({})", league.name(), league.season());
# Ok(())
# }
```

See the [workspace README](https://github.com/Sectorbob/fantasy-sports-rs/blob/main/README.md) and [architecture notes](https://github.com/Sectorbob/fantasy-sports-rs/blob/main/docs/ARCHITECTURE.md).
