//! Live tests against the real Yahoo API. They need your own credentials and are ignored by
//! default; run them with `cargo test -- --ignored` after exporting:
//!
//! - `YAHOO_CLIENT_ID`, `YAHOO_REDIRECT_URI` (your Yahoo developer app)
//! - `YAHOO_TOKEN_FILE` pointing at a token file produced by the auth flow
//!   (see `notes/README.md`)

use yahoo_fantasy_rs::{Client, GameCode};

#[tokio::test]
#[ignore = "live: needs Yahoo credentials and a token file"]
async fn live_get_games() {
    let client = Client::new_from_env()
        .expect("YAHOO_CLIENT_ID and YAHOO_REDIRECT_URI must be set")
        .with_token_file::<String>(None)
        .await
        .expect("failed to load the token file (YAHOO_TOKEN_FILE)");

    // only refreshes when the token is about to expire
    client.refresh_token(false).await.expect("failed to refresh token");

    let games = client
        .get_games(vec![GameCode::NFL], vec!["2019"])
        .await
        .expect("failed to fetch games");
    assert!(!games.is_empty(), "expected at least one NFL game for 2019");
}
