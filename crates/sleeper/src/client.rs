use std::{collections::HashMap, error::Error};

use crate::data::{Player, Sport};

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
                true => {
                    // panic!("{}", resp.text().await.unwrap())
                    match resp.json::<HashMap<String, Player>>().await {
                        Ok(players) => Ok(players),
                        // Err(json_error) => json_error.source()
                        Err(json_error) => {
                            println!("failed to parse json: {:#?}", json_error);
                            panic!("{}: {:?}", json_error, json_error.source())
                        }
                    }
                }
                false => panic!("got error response: {}", resp.status()),
            },
            Err(error) => Err(error),
        }
    }
}
