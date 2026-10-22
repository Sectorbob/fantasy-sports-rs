# Security

## Reporting a vulnerability

Please report security problems privately through GitHub's
[private vulnerability reporting](https://github.com/Sectorbob/fantasy-sports-rs/security/advisories/new)
rather than a public issue. I will acknowledge a report within a few days.

## Handling of credentials

- Nothing in this repository contains credentials. The Yahoo client is an OAuth2 *public* client (PKCE),
  so it has no client secret; sectorbot reads its Discord token and API keys from the environment.
- Yahoo tokens are stored in a JSON file you choose; keep it out of version control (`.token.json` is
  ignored by default) and readable only by you.
- The CI scans the full history with [gitleaks](https://github.com/gitleaks/gitleaks) on every push.

## Known advisories

`cargo audit` reports advisories for rustls 0.22 and rustls-webpki 0.102, which `serenity` 0.12 (used only
by `sectorbot`) depends on, and for two unmaintained crates used by `sled`. They are listed with their
reasons in [`deny.toml`](deny.toml) and are reviewed whenever those dependencies are updated.
