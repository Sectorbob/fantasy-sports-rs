use crate::{
    BoxScore, Draftboard, FetchRostersResponse, LeagueActivityItem, LeagueRules, LeagueStandings,
    LeaguesForUserResponse, ResponseList, Sport,
};
use fantasy_sports_cache::{Error, RequestContext, ResponseBodyFormat};
use std::path::PathBuf;

const DEFAULT_BASE_URL: &str = "https://www.fleaflicker.com/api";

pub struct Client {
    client: fantasy_sports_cache::Client,
}
impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Client {
            client: fantasy_sports_cache::Client::new(DEFAULT_BASE_URL.to_string()),
        }
    }

    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.client = self.client.with_base_url(base_url);
        self
    }

    pub async fn with_cache<T: Into<PathBuf>>(
        mut self,
        cache_dir: T,
    ) -> Result<Self, fantasy_sports_cache::Error> {
        self.client = self.client.with_cache(cache_dir)?;
        Ok(self)
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueActivity".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchUserLeagues".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueTransactions".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueRosters".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueBoxscore".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueDraftBoard".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
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

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueRules".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
    }

    pub async fn fetch_league_standings(
        &self,
        league_id: i32,
        sport: Option<Sport>,
        season: Option<i32>,
    ) -> Result<LeagueStandings, Error> {
        let mut filters = vec![("league_id", league_id.to_string())];
        if let Some(sport) = sport {
            filters.push(("sport", sport.to_string()));
        }
        if let Some(season) = season {
            filters.push(("season", season.to_string()));
        }

        self.client
            .build_request(
                RequestContext {
                    url: "/FetchLeagueStandings".to_string(),
                    filters: Some(filters),
                },
                false,
                Some(ResponseBodyFormat::JSON),
            )
            .send()
            .await
    }

    pub fn request_count(&self) -> u64 {
        self.client.request_count()
    }
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
            .and(path("/FetchUserLeagues".to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .get_leagues_for_user("user@example.com", None, None)
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
            .and(path("/FetchUserLeagues".to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let response = client
            .get_leagues_for_user("user@example.com", None, None)
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
            .and(path("/FetchLeagueActivity".to_string()))
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
            .and(path("/FetchLeagueActivity".to_string()))
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
            .and(path("/FetchLeagueTransactions".to_string()))
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
            .and(path("/FetchLeagueRosters".to_string()))
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
            .and(path("/FetchLeagueBoxscore".to_string()))
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
            .and(path("/FetchLeagueRules".to_string()))
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
            .and(path("/FetchLeagueDraftBoard".to_string()))
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
            .and(path("/FetchLeagueDraftBoard".to_string()))
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

    #[tokio::test]
    async fn test_fetch_league_standings_2025() {
        let league_id = 348896;
        let body = fs::read_to_string("testdata/fetch_league_standings_pre.json")
            .await
            .expect("failed to load test data");

        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path("/FetchLeagueStandings".to_string()))
            .and(query_param("league_id", league_id.to_string()))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let league_standings = client
            .fetch_league_standings(league_id, None, None)
            .await
            .expect("request failed");

        debug_assert_eq!(2025, league_standings.season, "expected league season");
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
    //         .get("/FetchUserLeagues?email=user@example.com")
    //         .await
    //         .expect("request failed");

    //     let code = response.status();
    //     println!("Status Code: {code}");

    //     let text = response.text().await.expect("failed to read response body");
    //     println!("Body:");
    //     println!("{text}");
    // }
}
