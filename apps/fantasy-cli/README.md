# fantasy-cli

A terminal UI for browsing your fantasy football leagues across platforms, built on the
[`fantasy-sports`](../../crates/fantasy-sports) crates with `ratatui`.

```shell
cargo run -p fantasy-sports-cli -- tui
```

It lists your leagues (Sleeper, Yahoo) for the seasons it scans, and from a league you can open
the summary, matchups, the draft board, transactions and a multi-season history of owner records.
Data is fetched in the background and cached under your OS cache directory, so the second run is
fast.

| Key | Action |
|---|---|
| `q` | quit |
| `l` | league list |
| `w` | welcome page |
| `b` | back |
| `↑` / `↓`, `Enter` | league list: move, open the league summary |
| `r` | league list: owner records across seasons |
| `m` / `t` / `d` | league summary: matchups / transactions / draft board |
| `r` | league summary: refresh the league from the network |

## Configuration

`config.json` in your OS config directory (`~/.config/fantasy-cli/config.json` on Linux; it is
created with defaults on first run). See [`config.example.json`](config.example.json):

| Field | Meaning |
|---|---|
| `landing_page` | `WelcomePage`, `LeaguesPage` or `{"LeagueSummary": "sleeper:<league id>"}` |
| `sleeper_user_id` | Your Sleeper user id; Sleeper leagues are skipped without it |
| `history.league_name` | Only leagues with this name appear on the history page (all if unset) |
| `history.owners` | People who appear under several ids over the years (Sleeper user ids, Yahoo guids); their records are merged |

Yahoo needs `YAHOO_CLIENT_ID` and `YAHOO_REDIRECT_URI` and a token file; see
[`fantasy-sports-yahoo`](../../crates/yahoo/README.md).

## Developer commands

`cargo run -p fantasy-sports-cli -- sleeper <league id>...` and `-- yahoo` run command line
exploration harnesses instead of the UI.

```shell
cargo test -p fantasy-sports-cli
```
