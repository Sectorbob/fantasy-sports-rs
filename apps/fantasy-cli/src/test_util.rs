//! Helpers for rendering tests: draw into an in-memory terminal and read back the text.
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("test backend never fails")
}

/// The buffer as lines of text, one per row, trailing spaces trimmed.
pub fn screen(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Whether any row of the screen contains `needle`.
pub fn contains(buffer: &Buffer, needle: &str) -> bool {
    screen(buffer).iter().any(|row| row.contains(needle))
}

// ---------------------------------------------------------------------------------------------
// Real leagues for the page tests: the sleeper crate's recorded (pseudonymized) API responses are
// replayed by a mock server and fetched through the same LeagueAccessor the app uses.

use fantasy_sports_core as core;
use fantasy_sports_yahoo as yahoo;
use ratatui::Frame;
use std::path::PathBuf;
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{method, path_regex},
};

pub const LEAGUE_ID: &str = "164974877262287888";
/// the user who owns roster 1 in that league, as found in the recorded users
pub const DRAFT_ID: &str = "685822547406541191";
/// a user who played in that league (it is returned for them in the 2019 season)
pub const USER_ID: &str = "727598467304811923";

fn sleeper_testdata(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/sleeper/testdata")
        .join(rel)
}

struct SleeperFixtures;
impl Respond for SleeperFixtures {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        let p = req.url.path();
        let league = format!("/league/{LEAGUE_ID}");
        let json = |rel: String| match std::fs::read_to_string(sleeper_testdata(&rel)) {
            Ok(body) => ResponseTemplate::new(200).set_body_raw(body, "application/json"),
            Err(_) => ResponseTemplate::new(404),
        };
        if p == league {
            json(format!("v1/league/{LEAGUE_ID}.json"))
        } else if let Some(rest) = p.strip_prefix(&format!("{league}/")) {
            match rest {
                "users" | "rosters" | "drafts" | "traded_picks" | "winners_bracket"
                | "losers_bracket" => json(format!("v1/league/{LEAGUE_ID}/{rest}.json")),
                "matchups/1" => json("league_matchups_1.json".to_string()),
                _ => ResponseTemplate::new(200).set_body_string("[]"),
            }
        } else if p == format!("/draft/{DRAFT_ID}/picks") {
            json(format!("v1/draft/{DRAFT_ID}/picks.json"))
        } else if p == format!("/draft/{DRAFT_ID}") {
            json(format!("v1/draft/{DRAFT_ID}.json"))
        } else if p == "/players/nfl" {
            ResponseTemplate::new(200)
                .set_body_raw(
                    std::fs::read(sleeper_testdata("players_nfl.json.gz")).unwrap(),
                    "application/json",
                )
                .insert_header("Content-Encoding", "gzip")
        } else if p == format!("/user/{USER_ID}/leagues/nfl/2019") {
            // the recorded league, listed as one of this user's leagues
            let league =
                std::fs::read_to_string(sleeper_testdata(&format!("v1/league/{LEAGUE_ID}.json")))
                    .unwrap();
            ResponseTemplate::new(200).set_body_raw(format!("[{league}]"), "application/json")
        } else if p.starts_with("/user/") && p.contains("/leagues/") {
            let rel = format!("v1{p}.json");
            if sleeper_testdata(&rel).exists() {
                json(rel)
            } else {
                ResponseTemplate::new(200).set_body_string("[]")
            }
        } else {
            ResponseTemplate::new(404)
        }
    }
}

pub async fn sleeper_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(".*"))
        .respond_with(SleeperFixtures)
        .mount(&server)
        .await;
    server
}

/// An accessor wired to the mock sleeper server, with yahoo faked (and never called) so the
/// result does not depend on the developer's environment.
pub async fn accessor(server: &MockServer) -> core::LeagueAccessor {
    let mut accessor = core::LeagueAccessor::new()
        .with_sleeper_base_url(server.uri())
        .with_yahoo_base_url(server.uri())
        .with_yahoo_settings(
            "client",
            "redirect",
            None::<String>,
            Some(yahoo::Token {
                access_token: "token".into(),
                token_type: "bearer".into(),
                expires_in: 3600,
                refresh_token: "refresh".into(),
                scope: None,
                ts: Some(chrono::Utc::now()),
            }),
        );
    accessor.init().await.expect("the accessor starts");
    accessor
}

pub async fn sleeper_league() -> core::League {
    let server = sleeper_server().await;
    accessor(&server)
        .await
        .fetch_league_context(
            &core::ExternalId::new(core::Platform::Sleeper, LEAGUE_ID.to_string()),
            false,
        )
        .await
        .expect("the recorded league loads")
}

/// Runs `draw` against a `width` x `height` in-memory screen and returns what ended up on it.
pub fn render(width: u16, height: u16, draw: impl FnOnce(&mut Frame<'_>)) -> Buffer {
    let mut terminal = terminal(width, height);
    terminal.draw(draw).expect("test backend never fails");
    terminal.backend().buffer().clone()
}

pub fn text(buffer: &Buffer) -> String {
    screen(buffer).join("\n")
}

/// Like [`accessor`] but with no yahoo configured at all (unless the developer's environment
/// configures it, in which case callers should skip).
pub async fn accessor_without_yahoo(server: &MockServer) -> Option<core::LeagueAccessor> {
    if std::env::var("YAHOO_CLIENT_ID").is_ok() {
        return None;
    }
    let mut accessor = core::LeagueAccessor::new().with_sleeper_base_url(server.uri());
    accessor.init().await.expect("the accessor starts");
    Some(accessor)
}
