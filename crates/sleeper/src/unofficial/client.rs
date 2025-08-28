use crate as core;
use reqwest::header::HeaderMap;

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0";
const UNOFFICIAL_API_BASE: &str = "https://api.sleeper.app";

pub struct Client {
    cli: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new() -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Accept", "application/json".parse().unwrap());
        default_headers.insert("Accept-Encoding", "gzip".parse().unwrap());
        default_headers.insert("Connection", "keep-alive".parse().unwrap());
        default_headers.insert("DNT", "1".parse().unwrap());
        // If-None-Match: W/"fd83bb01b59c8401063c771a6b341f53"
        // Priority: u=0, i
        Client {
            cli: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(default_headers)
                // .tcp_keepalive()
                .gzip(true)
                .build()
                .expect("failed to build reqwest client"),
            base_url: UNOFFICIAL_API_BASE.to_string(),
        }
    }

    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Get player projections for a given sport, season, week, and season type.
    ///
    /// NBA goes back to 2020
    /// NFL goes back to 2018
    pub async fn projections(
        &self,
        sport: &core::Sport,
        season: i16,
        week: u8,
        season_type: &core::SeasonType,
    ) -> Result<Vec<core::unofficial::Projection>, reqwest::Error> {
        let url = format!(
            "{base_url}/projections/{sport}/{season}/{week}",
            base_url = self.base_url
        );
        let resp = self
            .cli
            .get(&url)
            .query(&[("season_type", season_type.to_string())])
            .send()
            .await?;
        resp.json().await
    }

    /// Get player stats for a given sport, season, week, and season type.
    ///
    /// NBA goes back to 2020
    /// NFL goes back to 2015
    pub async fn stats(
        &self,
        sport: &core::Sport,
        season: i16,
        week: u8,
        season_type: &core::SeasonType,
    ) -> Result<Vec<core::unofficial::Statline>, reqwest::Error> {
        let url = format!(
            "{base_url}/stats/{sport}/{season}/{week}",
            base_url = self.base_url
        );
        let resp = self
            .cli
            .get(&url)
            .query(&[("season_type", season_type.to_string())])
            .send()
            .await?;
        resp.json().await
    }
}

#[cfg(test)]
mod tests {
    use crate as core;
    use core::unofficial::Client;
    use tokio::fs;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };
    #[tokio::test]
    async fn test_get_2025_wk1_projections() {
        let (mock_server, client) = setup().await;
        let json_body = fs::read_to_string("testdata/unofficial/projections/nfl/2025/1.json")
            .await
            .expect("unable to read test data file");
        Mock::given(method("GET"))
            .and(path(format!("/projections/nfl/2025/1")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let projections = client
            .projections(&core::Sport::NFL, 2025, 1, &core::SeasonType::Regular)
            .await
            .expect("unable to fetch projections");
        assert!(!projections.is_empty());
    }

    #[tokio::test]
    async fn test_get_2023_wk1_stats() {
        let (mock_server, client) = setup().await;
        let json_body = fs::read_to_string("testdata/unofficial/stats/nfl/2023/1.json")
            .await
            .expect("unable to read test data file");
        Mock::given(method("GET"))
            .and(path(format!("/stats/nfl/2023/1")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let stats = client
            .stats(&core::Sport::NFL, 2023, 1, &core::SeasonType::Regular)
            .await
            .expect("unable to fetch stats");
        assert!(!stats.is_empty());
    }

    // #[tokio::test]
    // async fn test_crawl_seasons() {
    //     let seasons = 2020..2025;
    //     let weeks = 1..17;

    //     let client = Client::new();

    //     for season in seasons {
    //         for week in weeks.clone() {
    //             println!("{season}.{week}:");
    //             let stats = client
    //                 .projections(&core::Sport::NFL, season, week, &core::SeasonType::Regular)
    //                 .await
    //                 .expect("unable to fetch stats");
    //         }
    //     }
    // }

    async fn setup() -> (MockServer, Client) {
        let mock_server = MockServer::start().await;
        let client = Client::new().with_base_url(mock_server.uri());
        (mock_server, client)
    }
}
