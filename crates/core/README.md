# fantasy-sports-core

The platform independent data model for fantasy sports leagues, built over the
[Sleeper](../sleeper), [Yahoo](../yahoo) and [Fleaflicker](../fleaflicker) clients.

- `LeagueAccessor` fetches a whole league (settings, rosters, drafts, matchups, transactions,
  standings) from any platform given an `ExternalId` such as `sleeper:1234567890`, and caches it.
- `League` is an enum with a variant per platform and common accessors for what they share.
- `PlayerCache` maps Yahoo player ids to Sleeper's player universe so rosters look alike.
- `history::build_owner_records` merges one person's records across seasons and platforms.

Most applications should depend on the [`fantasy-sports`](../fantasy-sports) facade instead of
this crate directly.

```shell
cargo test -p fantasy-sports-core
```
