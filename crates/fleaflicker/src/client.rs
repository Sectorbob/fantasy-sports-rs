use crate::{Sport, data};
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
    ) -> Result<data::LeagueActivityResponse, Error> {
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
    ) -> Result<data::LeaguesForUserResponse, Error> {
        let email = email.into();
        return self
            .get("/FetchUserLeagues", Some(vec![("email", email)]))
            .await;
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
        matchers::{method, path},
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
            .get_leagues_for_user("user_a271a94dea@example.com")
            .await
            .expect("request failed");

        debug_assert_eq!(2025, response.season, "season is incorrect");
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

        debug_assert_eq!(7, response.result_total, "result_total is incorrect");
        debug_assert_eq!(7, response.items.len(), "item count incorrect");
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
