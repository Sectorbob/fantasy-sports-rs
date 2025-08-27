mod data;
use crate::{SeasonType, Sport};
use data::*;

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0";
const UNOFFICIAL_API_BASE: &str = "https://api.sleeper.app";

pub struct Client {
    cli: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new() -> Self {
        Client {
            cli: reqwest::Client::builder()
                .user_agent(USER_AGENT)
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

    pub async fn projections<T: Into<String>>(
        &self,
        sport: &Sport,
        season: T,
        week: usize,
        season_type: &SeasonType,
    ) -> Result<Vec<Statline>, reqwest::Error> {
        let season = season.into();
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

    pub async fn stats<T: Into<String>>(
        &self,
        sport: &Sport,
        season: T,
        week: usize,
        season_type: &SeasonType,
    ) -> Result<Vec<Statline>, reqwest::Error> {
        let season = season.into();
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
    use std::collections::HashSet;

    use tokio::fs;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::unofficial::{Client, data::Statline};

    #[test]
    fn deserialize_2023_projections() {
        let data = include_str!("../../testdata/unofficial/projections/nfl/2023/1.json");
        let projections: Vec<Statline> = serde_json::from_str(data).expect("unable to deserialize");
        assert!(!projections.is_empty());
        debug_assert_eq!(8622, projections.len(), "should have 8622 total proj");
        // let filtered: Vec<&Statline> = projections.iter().filter(|p| p.game_id.is_some()).collect();
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.game_id.is_none()).count(),
            "should have 7862 proj missing a game_id" // wtf
        );
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.date.is_none()).count(),
            "should have 7862 proj missing a date" // wtf
        );
        debug_assert_eq!(
            8622,
            projections.iter().filter(|p| p.status.is_none()).count(),
            "all projections should have no status" // wtf
        );
        debug_assert_eq!(
            5702,
            projections.iter().filter(|p| p.team.is_none()).count(),
            "should have 5702 proj missing a team" // wtf
        );
        debug_assert_eq!(
            7862,
            projections.iter().filter(|p| p.opponent.is_none()).count(),
            "should have 7862 proj missing an opponent" // wtf
        );

        let statuses: HashSet<String> = projections
            .iter()
            .filter_map(|p| p.status.clone())
            .collect();
        debug_assert_eq!(HashSet::new(), statuses, "should have correct statuses");
    }

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
            .projections(&super::Sport::NFL, "2025", 1, &super::SeasonType::Regular)
            .await
            .expect("unable to fetch projections");
        assert!(!projections.is_empty());
    }

    async fn setup() -> (MockServer, Client) {
        let mock_server = MockServer::start().await;
        let client = Client::new().with_base_url(mock_server.uri());
        (mock_server, client)
    }
}
