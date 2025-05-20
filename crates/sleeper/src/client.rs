use std::{collections::HashMap, error::Error};

use crate::data::{Player, Sport, TrendingPlayer};

const SLEEPER_API_BASE_URL: &str = "https://api.sleeper.app/v1";
const SLEEPER_CDN_BASE_URL: &str = "https://sleepercdn.com";

pub struct Client {
    client: reqwest::Client,
}
impl Client {
    pub fn new() -> Self {
        Client {
            client: reqwest::ClientBuilder::new()
                .build()
                .expect("dev error while building http client"),
        }
    }

    pub async fn fetch_all_players(
        &self,
        sport: Sport,
    ) -> Result<HashMap<String, Player>, reqwest::Error> {
        match self
            .client
            .get(format!("{}/players/{}", SLEEPER_API_BASE_URL, sport))
            .send()
            .await
        {
            Ok(resp) => match resp.status().is_success() {
                true => match resp.json::<HashMap<String, Player>>().await {
                    Ok(players) => Ok(players),
                    Err(json_error) => {
                        println!("failed to parse json: {:#?}", json_error);
                        panic!("{}: {:?}", json_error, json_error.source())
                    }
                },
                false => panic!("got error response: {}", resp.status()),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn get_trending_players(
        &self,
        sport: Sport,
        _type: String,
        lookback_hours: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<TrendingPlayer>, reqwest::Error> {
        let mut req = self.client.get(format!(
            "{}/players/{}/trending/{}",
            SLEEPER_API_BASE_URL, sport, _type
        ));
        if let Some(lookback_hours) = lookback_hours {
            req = req.query(&[("lookback_hours", lookback_hours.to_string().as_str())]);
        }
        if let Some(limit) = limit {
            req = req.query(&[("limit", limit.to_string().as_str())]);
        }

        match req.send().await {
            Ok(resp) => match resp.status().is_success() {
                true => match resp.json::<Vec<TrendingPlayer>>().await {
                    Ok(players) => Ok(players),
                    Err(json_error) => {
                        println!("failed to parse json: {:#?}", json_error);
                        panic!("{}: {:?}", json_error, json_error.source())
                    }
                },
                false => panic!("got error response: {}", resp.status()),
            },
            Err(error) => Err(error),
        }
    }
}
