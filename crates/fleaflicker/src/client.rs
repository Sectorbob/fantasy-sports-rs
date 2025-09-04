use crate::{
    BoxScore, Draftboard, FetchRostersResponse, LeagueActivityItem, LeagueRules,
    LeaguesForUserResponse, ResponseList, Sport,
};
use serde::de::DeserializeOwned;

const DEFAULT_BASE_URL: &str = "https://www.fleaflicker.com/api";

pub struct Client {
    cli: reqwest::Client,
    base_url: String,
}
impl Client {
    pub fn new() -> Self {
        Client {
            cli: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub async fn get<P: Into<String>, T: DeserializeOwned>(
        &self,
        path: P,
        filters: Option<Vec<(&'static str, String)>>,
    ) -> Result<T, Error> {
        let req = self
            .build_get_json_request(path.into(), filters)
            .expect("failed to build request");
        log::debug!("GET {url}", url = req.url());
        Ok(self.cli.execute(req).await?.json().await?)
    }

    pub(crate) fn build_get_json_request(
        &self,
        path: String,
        filters: Option<Vec<(&'static str, String)>>,
    ) -> Result<reqwest::Request, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);

        let mut request_builder = self.cli.get(&url).header("Accept", "application/json");

        if let Some(filters) = filters {
            request_builder = request_builder.query(&filters)
        }
        request_builder.build()
    }

    pub async fn fetch_league_activity(
        &self,
        sport: &Sport,
        league_id: i32,
        result_offset: Option<i32>,
    ) -> Result<ResponseList<LeagueActivityItem>, Error> {
        let mut filters = vec![
            ("sport", sport.to_string()),
            ("league_id", league_id.to_string()),
        ];
        if let Some(result_offset) = result_offset {
            filters.push(("result_offset", result_offset.to_string()));
        }

        return self.get("/FetchLeagueActivity", Some(filters)).await;
    }

    pub async fn get_leagues_for_user<T: Into<String>>(
        &self,
        email: T,
        sport: Option<Sport>,
        season: Option<u32>,
    ) -> Result<LeaguesForUserResponse, Error> {
        let mut filters = vec![("email", email.into())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(season) = season {
            filters.push(("season", season.to_string()));
        }

        return self.get("/FetchUserLeagues", Some(filters)).await;
    }

    pub async fn get_transaction_history(
        &self,
        league_id: i32,
        sport: Option<Sport>,
        team_id: Option<i32>,
        result_offset: Option<i32>,
    ) -> Result<ResponseList<LeagueActivityItem>, Error> {
        let mut filters = vec![("league_id", league_id.to_string())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(team_id) = team_id {
            filters.push(("team_id", team_id.to_string()));
        }
        if let Some(result_offset) = result_offset {
            filters.push(("result_offset", result_offset.to_string()));
        }

        return self.get("/FetchLeagueTransactions", Some(filters)).await;
    }

    pub async fn fetch_league_rosters(
        &self,
        league_id: i32,
        sport: Option<Sport>,
        season: Option<i32>,
        scoring_period: Option<i32>,
        external_id_type: Option<Vec<String>>,
    ) -> Result<FetchRostersResponse, Error> {
        let mut filters = vec![("league_id", league_id.to_string())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(season) = season {
            filters.push(("season", season.to_string()));
        }
        if let Some(scoring_period) = scoring_period {
            filters.push(("scoring_period", scoring_period.to_string()));
        }
        if let Some(external_id_type) = external_id_type {
            for entry in external_id_type {
                filters.push(("external_id_type", entry));
            }
        }

        return self.get("/FetchLeagueRosters", Some(filters)).await;
    }

    pub async fn fetch_league_boxscore(
        &self,
        league_id: i32,
        sport: Option<Sport>,
        fantasy_game_id: i64,
        scoring_period: Option<i32>,
    ) -> Result<BoxScore, Error> {
        let mut filters = vec![
            ("league_id", league_id.to_string()),
            ("fantasy_game_id", fantasy_game_id.to_string()),
        ];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(scoring_period) = scoring_period {
            filters.push(("scoring_period", scoring_period.to_string()));
        }

        return self.get("/FetchLeagueBoxscore", Some(filters)).await;
    }

    pub async fn fetch_league_draftboard(
        &self,
        league_id: i32,
        sport: Option<Sport>,
        season: Option<i32>,
        draft_number: Option<u8>,
        external_id_type: Option<Vec<String>>,
    ) -> Result<Draftboard, Error> {
        let mut filters = vec![("league_id", league_id.to_string())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(season) = season {
            filters.push(("season", season.to_string()));
        }
        if let Some(draft_number) = draft_number {
            filters.push(("draft_number", draft_number.to_string()));
        }
        if let Some(external_id_type) = external_id_type {
            for entry in external_id_type {
                filters.push(("external_id_type", entry));
            }
        }

        return self.get("/FetchLeagueDraftBoard", Some(filters)).await;
    }

    pub async fn fetch_league_rules(
        &self,
        league_id: i32,
        sport: Option<Sport>,
    ) -> Result<LeagueRules, Error> {
        let mut filters = vec![("league_id", league_id.to_string())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }

        return self.get("/FetchLeagueRules", Some(filters)).await;
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("json deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use crate::Client;
    use tokio::fs;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path, query_param},
    };

    #[tokio::test]
    async fn test_get_leagues_for_user() {
        let body = fs::read_to_string("testdata/leagues_for_user.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchUserLeagues")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .get_leagues_for_user("user_a271a94dea@example.com", None, None)
            .await
            .expect("request failed");

        debug_assert_eq!(2025, response.season, "season is incorrect");
        debug_assert_eq!(1, response.leagues.len(), "league count incorrect");
    }

    #[tokio::test]
    async fn test_get_leagues_for_user_2019() {
        let body = fs::read_to_string("testdata/leagues_for_user_2019.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchUserLeagues")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .get_leagues_for_user("user_a271a94dea@example.com", None, None)
            .await
            .expect("request failed");

        debug_assert_eq!(2019, response.season, "season is incorrect");
        debug_assert_eq!(1, response.leagues.len(), "league count incorrect");
    }

    #[tokio::test]
    async fn test_fetch_league_activity() {
        let body = fs::read_to_string("testdata/league_activity.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueActivity")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .fetch_league_activity(&crate::Sport::NFL, 348824, None)
            .await
            .expect("request failed");

        debug_assert_eq!(Some(7), response.result_total, "result_total is incorrect");
        debug_assert_eq!(7, response.items.len(), "item count incorrect");
    }

    #[tokio::test]
    async fn test_fetch_league_activity_2019() {
        let league_id = 156377;
        let body = fs::read_to_string("testdata/league_activity_2019.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueActivity")))
            .and(query_param("league_id", league_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .fetch_league_activity(&crate::Sport::NFL, league_id, None)
            .await
            .expect("request failed");

        debug_assert_eq!(Some(29), response.result_total, "result_total is incorrect");
        debug_assert_eq!(29, response.items.len(), "item count incorrect");
    }

    #[tokio::test]
    async fn test_fetch_league_transactions_page_1() {
        let league_id = 156377;
        let body = fs::read_to_string("testdata/league_transactions_page_1.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueTransactions")))
            .and(query_param("league_id", league_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .get_transaction_history(league_id, None, None, None)
            .await
            .expect("request failed");
        // println!("{response:#?}");
        debug_assert_eq!(None, response.result_total, "result_total incorrect");
        debug_assert_eq!(
            Some(30),
            response.result_offset_next,
            "result_offset_next incorrect"
        );
        debug_assert_eq!(30, response.items.len(), "item count incorrect");
    }

    #[tokio::test]
    async fn test_fetch_league_roster_2025() {
        let league_id = 348896;
        let body = fs::read_to_string("testdata/league_rosters.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueRosters")))
            .and(query_param("league_id", league_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .fetch_league_rosters(league_id, None, None, None, None)
            .await
            .expect("request failed");

        debug_assert_eq!(12, response.rosters.len(), "should be 12 rosters");

        for (index, roster) in response.rosters.iter().enumerate() {
            println!("Roster: {:?}", roster.team.name);
            debug_assert!(
                roster.players.len() > 40,
                "roster {index} does not have at least 40 players"
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_league_boxscore_2025_1() {
        let league_id = 348896;
        let fantasy_game_id = 56760292;
        let body = fs::read_to_string("testdata/fetch_league_boxscore_1.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueBoxscore")))
            .and(query_param("league_id", league_id.to_string()))
            .and(query_param("fantasy_game_id", fantasy_game_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let boxscore = client
            .fetch_league_boxscore(league_id, None, fantasy_game_id, None)
            .await
            .expect("request failed");

        debug_assert_eq!(
            fantasy_game_id.to_string(),
            boxscore.game.id,
            "game matchup id is incorrect"
        );

        // println!("{boxscore:#?}");
        // assert!(false);
    }

    #[tokio::test]
    async fn test_fetch_league_rules_2025() {
        let league_id = 348896;
        let body = fs::read_to_string("testdata/fetch_league_rules.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueRules")))
            .and(query_param("league_id", league_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let rules = client
            .fetch_league_rules(league_id, None)
            .await
            .expect("request failed");

        debug_assert_eq!(22, rules.num_starters, "expected num_starters");
        debug_assert_eq!(45, rules.max_active, "expected max_active");
        debug_assert_eq!(60, rules.max_roster_size, "expected max_roster_size");
    }

    #[tokio::test]
    async fn test_fetch_league_draftboard_2025_0() {
        let league_id = 348896;
        let draft_number = 0;
        let body = fs::read_to_string("testdata/fetch_league_draftboard_0.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueDraftBoard")))
            .and(query_param("league_id", league_id.to_string()))
            .and(query_param("draft_number", draft_number.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let draftboard = client
            .fetch_league_draftboard(league_id, None, None, Some(draft_number), None)
            .await
            .expect("request failed");

        debug_assert_eq!(draft_number, draftboard.draft_index, "expected draft index");
        debug_assert_eq!(
            480,
            draftboard.ordered_selections.len(),
            "expected x draft selections"
        )
    }

    #[tokio::test]
    async fn test_fetch_league_draftboard_2025_1() {
        let league_id = 348896;
        let draft_number = 1;
        let body = fs::read_to_string("testdata/fetch_league_draftboard_1.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/FetchLeagueDraftBoard")))
            .and(query_param("league_id", league_id.to_string()))
            .and(query_param("draft_number", draft_number.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let draftboard = client
            .fetch_league_draftboard(league_id, None, None, Some(draft_number), None)
            .await
            .expect("request failed");

        debug_assert_eq!(draft_number, draftboard.draft_index, "expected draft index");
        debug_assert_eq!(
            56,
            draftboard.ordered_selections.len(),
            "expected x draft selections"
        )
    }

    async fn setup() -> (MockServer, Client) {
        let mock_server = MockServer::start().await;
        let client = Client::new().with_base_url(mock_server.uri());
        (mock_server, client)
    }

    // #[tokio::test]
    // async fn test_it() {
    //     let fixture = Client::new();

    //     let response = fixture
    //         .get("/FetchUserLeagues?email=user_a271a94dea@example.com")
    //         .await
    //         .expect("request failed");

    //     let code = response.status();
    //     println!("Status Code: {code}");

    //     let text = response.text().await.expect("failed to read response body");
    //     println!("Body:");
    //     println!("{text}");
    // }
}
