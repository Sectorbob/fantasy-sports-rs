//! The OAuth2 authorization-code-with-PKCE flow, against a mock of Yahoo's login service.
use crate::{Client, Error, Token};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tempfile::TempDir;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const TOKEN_JSON: &str = r#"{
    "access_token": "new-access",
    "refresh_token": "new-refresh",
    "expires_in": 3600,
    "token_type": "bearer"
}"#;

fn client(server: &MockServer) -> Client {
    Client::new("my-client-id", "https://example.invalid/callback").with_auth_base_url(server.uri())
}

fn token(expires_in: i64, age: Duration) -> Token {
    Token {
        access_token: "old-access".into(),
        token_type: "bearer".into(),
        expires_in,
        refresh_token: "old-refresh".into(),
        scope: None,
        ts: Some(Utc::now() - age),
    }
}

fn form(body: &[u8]) -> HashMap<String, String> {
    reqwest::Url::parse(&format!(
        "http://localhost/?{}",
        String::from_utf8_lossy(body)
    ))
    .unwrap()
    .query_pairs()
    .map(|(k, v)| (k.into_owned(), v.into_owned()))
    .collect()
}

async fn redirecting_auth_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/request_auth"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("location", "https://login.example.invalid/consent?x=1"),
        )
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn the_authorization_url_carries_the_client_and_a_pkce_challenge() {
    let server = redirecting_auth_server().await;

    let url = client(&server).get_request_auth_url().await.unwrap();

    assert_eq!("https://login.example.invalid/consent?x=1", url);
    let requests = server.received_requests().await.unwrap();
    let query: HashMap<String, String> = requests[0]
        .url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    assert_eq!("my-client-id", query["client_id"]);
    assert_eq!("https://example.invalid/callback", query["redirect_uri"]);
    assert_eq!("code", query["response_type"]);
    assert_eq!("S256", query["code_challenge_method"]);
    assert!(
        query["code_challenge"].len() >= 43,
        "a SHA-256 challenge is 43 characters"
    );
}

#[tokio::test]
async fn a_response_that_is_not_a_redirect_is_an_error() {
    let server = MockServer::start().await;
    Mock::given(path("/request_auth"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let err = client(&server).get_request_auth_url().await.unwrap_err();
    assert!(err.to_string().contains("unexpected response"), "{err}");
}

#[tokio::test]
async fn a_redirect_without_a_location_is_an_error_not_a_panic() {
    let server = MockServer::start().await;
    Mock::given(path("/request_auth"))
        .respond_with(ResponseTemplate::new(302))
        .mount(&server)
        .await;
    let err = client(&server).get_request_auth_url().await.unwrap_err();
    assert!(err.to_string().contains("without a location"), "{err}");
}

#[tokio::test]
async fn the_code_is_exchanged_with_the_verifier_that_matches_the_challenge() {
    let server = redirecting_auth_server().await;
    Mock::given(method("POST"))
        .and(path("/get_token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TOKEN_JSON, "application/json"))
        .mount(&server)
        .await;
    let dir = TempDir::new().unwrap();
    let token_file = dir.path().join("token.json").to_string_lossy().to_string();
    let mut client = client(&server);
    client.token_file = Some(token_file.clone());

    client.get_request_auth_url().await.unwrap();
    let token = client.get_token("the-code").await.unwrap();

    assert_eq!("new-access", token.access_token);
    assert!(token.ts.is_some(), "the time of issue is recorded");
    assert_eq!(
        "new-access",
        client.token().await.unwrap().unwrap().access_token
    );

    let requests = server.received_requests().await.unwrap();
    let challenge = requests[0]
        .url
        .query_pairs()
        .find(|(k, _)| k == "code_challenge")
        .map(|(_, v)| v.into_owned())
        .unwrap();
    let sent = form(&requests[1].body);
    assert_eq!("authorization_code", sent["grant_type"]);
    assert_eq!("the-code", sent["code"]);
    assert_eq!("my-client-id", sent["client_id"]);
    assert!(
        !sent.contains_key("client_secret"),
        "this is a public client"
    );
    // RFC 7636: the verifier is 43-128 unreserved characters, and the challenge is its S256 hash
    let verifier = &sent["code_verifier"];
    assert!((43..=128).contains(&verifier.len()), "{}", verifier.len());
    assert!(
        verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c))
    );
    assert_eq!(challenge, pkce::code_challenge(verifier.as_bytes()));

    // the token was saved for the next run
    let saved = crate::token::load_token_file(&token_file).await.unwrap();
    assert_eq!("new-refresh", saved.refresh_token);
}

#[tokio::test]
async fn exchanging_a_code_before_asking_for_authorization_is_an_error() {
    let server = MockServer::start().await;
    let err = client(&server).get_token("code").await.unwrap_err();
    assert!(err.to_string().contains("no code_verifier"), "{err}");
}

#[tokio::test]
async fn a_rejected_code_or_unreadable_token_is_an_error() {
    let server = redirecting_auth_server().await;
    let rejecting = client(&server);
    rejecting.get_request_auth_url().await.unwrap();
    Mock::given(path("/get_token"))
        .respond_with(ResponseTemplate::new(400).set_body_string("invalid_grant"))
        .mount(&server)
        .await;
    let err = rejecting.get_token("bad").await.unwrap_err();
    assert!(err.to_string().contains("400"), "{err}");

    let server = redirecting_auth_server().await;
    Mock::given(path("/get_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    let garbled = client(&server);
    garbled.get_request_auth_url().await.unwrap();
    let err = garbled.get_token("code").await.unwrap_err();
    assert!(err.to_string().contains("parse"), "{err}");
}

#[tokio::test]
async fn a_token_that_is_still_valid_is_not_refreshed() {
    let server = MockServer::start().await;
    let client = client(&server).with_token(token(3600, Duration::minutes(5)));

    let kept = client.refresh_token(false).await.unwrap();

    assert_eq!("old-access", kept.access_token);
    assert!(server.received_requests().await.unwrap().is_empty());
    assert_eq!(0, client.request_count());
}

#[tokio::test]
async fn an_expired_token_is_refreshed_and_saved() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/get_token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TOKEN_JSON, "application/json"))
        .mount(&server)
        .await;
    let dir = TempDir::new().unwrap();
    let token_file = dir.path().join("token.json").to_string_lossy().to_string();
    let mut client = client(&server).with_token(token(3600, Duration::hours(2)));
    client.token_file = Some(token_file.clone());

    let refreshed = client.refresh_token(false).await.unwrap();

    assert_eq!("new-access", refreshed.access_token);
    assert_eq!(
        "new-access",
        client.token().await.unwrap().unwrap().access_token
    );
    assert_eq!(1, client.request_count());
    let sent = form(&server.received_requests().await.unwrap()[0].body);
    assert_eq!("refresh_token", sent["grant_type"]);
    assert_eq!("old-refresh", sent["refresh_token"]);
    assert_eq!("my-client-id", sent["client_id"]);
    assert_eq!(
        "new-refresh",
        crate::token::load_token_file(&token_file)
            .await
            .unwrap()
            .refresh_token
    );
}

#[tokio::test]
async fn a_fresh_token_is_refreshed_when_forced() {
    let server = MockServer::start().await;
    Mock::given(path("/get_token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(TOKEN_JSON, "application/json"))
        .mount(&server)
        .await;
    let client = client(&server).with_token(token(3600, Duration::minutes(1)));

    let refreshed = client.refresh_token(true).await.unwrap();

    assert_eq!("new-access", refreshed.access_token);
}

#[tokio::test]
async fn refreshing_without_a_token_or_with_a_failing_server_is_an_error() {
    let server = MockServer::start().await;
    let err = client(&server).refresh_token(false).await.unwrap_err();
    assert!(err.to_string().contains("no token available"), "{err}");

    Mock::given(path("/get_token"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    let expired = client(&server).with_token(token(3600, Duration::hours(2)));
    let err = expired.refresh_token(false).await.unwrap_err();
    assert!(err.to_string().contains("500"), "{err}");
    // the old token is still there for another attempt
    assert_eq!(
        "old-access",
        expired.token().await.unwrap().unwrap().access_token
    );
}

#[tokio::test]
async fn an_unreachable_login_service_is_an_error_not_a_panic() {
    // port 1 is privileged and nothing listens on it, so the connection is refused (a freed
    // mock server port could be handed to another test, which would then see this request)
    let client = Client::new("id", "uri")
        .with_auth_base_url("http://127.0.0.1:1")
        .with_token(token(3600, Duration::hours(2)));

    let err: Error = client.refresh_token(false).await.unwrap_err();

    assert!(err.to_string().contains("refresh token request"), "{err}");
}
