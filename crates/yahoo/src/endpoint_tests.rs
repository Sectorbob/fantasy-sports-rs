//! Endpoints whose responses are derived from the recorded fixtures in `tests/data`.
use crate::{Client, Error, GameKey, LeagueKey, Token};
use chrono::Utc;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn fixture(rel: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/data/fantasy/v2/{rel}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

/// serves `body` at `url_path` and returns a client for it
async fn client_serving(url_path: &str, body: String) -> (Client, MockServer) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(url_path))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(body)
                .insert_header("Content-Type", "application/xml"),
        )
        .mount(&server)
        .await;
    let client = Client::new("", "")
        .with_base_url(server.uri())
        .with_token(Token {
            access_token: "token".into(),
            token_type: "bearer".into(),
            expires_in: 3600,
            refresh_token: "refresh".into(),
            scope: None,
            ts: Some(Utc::now()),
        });
    (client, server)
}

/// the first `<game>` of the recorded games response, as the root of its own response
fn single_game_response() -> String {
    let games = fixture("games;game_codes=nfl;seasons=2019.xml");
    let start = games.find("<game>").unwrap();
    let end = games.find("</game>").unwrap() + "</game>".len();
    let root_open = &games[games.find("<fantasy_content").unwrap()..games.find("<games").unwrap()];
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{root_open}{}</fantasy_content>",
        &games[start..end]
    )
}

#[tokio::test]
async fn a_single_game_is_fetched_by_its_key() {
    let (client, _server) = client_serving("/game/390", single_game_response()).await;

    let game = client.get_game(GameKey::Id(390)).await.unwrap();

    assert_eq!(390, game.game_id);
    assert_eq!("Football", game.name);
    assert_eq!("2019", game.season);
}

#[tokio::test]
async fn asking_for_a_game_and_getting_something_else_is_a_clear_error() {
    let (client, _server) =
        client_serving("/game/390", fixture("league/380.l.129397/teams/roster.xml")).await;

    let err = client.get_game(GameKey::Id(390)).await.unwrap_err();

    assert!(matches!(err, Error::UnexpectedContentVariant(_)), "{err}");
    assert!(
        err.to_string().contains("expected FantasyContent::Game"),
        "{err}"
    );
}

#[tokio::test]
async fn the_logged_in_user_is_fetched_with_their_games() {
    let (client, _server) = client_serving(
        "/users;use_login=1",
        fixture("users;use_login=1/games;game_keys=390/teams.xml"),
    )
    .await;

    let users = client.get_users(true).await.unwrap();

    assert_eq!(1, users.len());
    assert!(!users[0].guid.is_empty());
    assert_eq!(390, users[0].games[0].game_id);
}

#[tokio::test]
async fn users_without_use_login_ask_for_all_users() {
    let (client, server) = client_serving(
        "/users",
        fixture("users;use_login=1/games;game_keys=390/teams.xml"),
    )
    .await;

    client.get_users(false).await.unwrap();

    assert_eq!(
        "/users",
        server.received_requests().await.unwrap()[0].url.path()
    );
}

#[tokio::test]
async fn the_teams_of_a_league_are_listed() {
    let key: LeagueKey = "380.l.129397".parse().unwrap();
    let (client, _server) = client_serving(
        "/league/380.l.129397/teams",
        fixture("league/380.l.129397/teams/roster.xml"),
    )
    .await;

    let teams = client.get_teams_in_league(&key).await.unwrap();

    assert!(teams.len() >= 2, "{} teams", teams.len());
    assert!(teams.iter().all(|t| t.team_key.league_key == key));
}

#[tokio::test]
async fn a_failing_request_is_reported_with_the_status() {
    let server = MockServer::start().await;
    Mock::given(path("/game/1"))
        .respond_with(ResponseTemplate::new(500).set_body_string("<oops/>"))
        .mount(&server)
        .await;
    let client = Client::new("", "")
        .with_base_url(server.uri())
        .with_token(Token {
            access_token: "t".into(),
            token_type: "bearer".into(),
            expires_in: 3600,
            refresh_token: "r".into(),
            scope: None,
            ts: Some(Utc::now()),
        });

    let err = client.get_game(GameKey::Id(1)).await.unwrap_err();

    assert!(err.to_string().contains("500"), "{err}");
}

#[tokio::test]
async fn requests_without_a_token_fail_before_going_out() {
    let server = MockServer::start().await;
    let client = Client::new("", "").with_base_url(server.uri());

    let err = client.get_game(GameKey::Id(1)).await.unwrap_err();

    assert!(err.to_string().contains("no token available"), "{err}");
    assert!(server.received_requests().await.unwrap().is_empty());
}
