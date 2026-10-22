# fantasy-sports-yahoo

A Rust client for the Yahoo Fantasy Sports API: games, leagues (settings, standings, scoreboards,
draft results, transactions), teams and rosters, players. Responses are XML, parsed into typed
structures, and cached on disk through [`fantasy-sports-cache`](../cache).

## Authentication

Yahoo uses OAuth2. This client implements the authorization code flow with **PKCE as a public
client**, so no client secret is needed, only the client id and redirect uri of your
[Yahoo developer app](https://developer.yahoo.com/apps/):

```shell
export YAHOO_CLIENT_ID=...
export YAHOO_REDIRECT_URI=...
```

`Client::get_request_auth_url` produces the URL to send the user to, `Client::get_token` exchanges
the returned code for a token, and the token is stored in a JSON file (`with_token_file`).
`refresh_token(false)` refreshes it only when it has expired. A token that expires mid-run is
reported as `Error::TokenExpired`.

[`notes/`](notes) has curl scripts that walk through the same flow by hand.

## Status

Read access is implemented. Not yet implemented: write operations (roster changes,
transactions), the server-side flow with a client secret, an auth `state` parameter and
filters for the `/players` resource.

Live tests need real credentials and are ignored by default (`cargo test -- --ignored`).
