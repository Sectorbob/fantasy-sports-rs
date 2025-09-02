use crate::data;
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

    pub async fn get<P: Into<String>, T: DeserializeOwned>(&self, path: P) -> Result<T, Error> {
        Ok(self
            .cli
            .get(format!(
                "{base_url}{path}",
                base_url = self.base_url,
                path = path.into()
            ))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_leagues_for_user<T: Into<String>>(
        &self,
        email: T,
    ) -> Result<data::LeaguesForUserResponse, Error> {
        let email = email.into();
        return self.get(format!("/FetchUserLeagues?email={email}")).await;
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
    use tokio::fs;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::Client;

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
