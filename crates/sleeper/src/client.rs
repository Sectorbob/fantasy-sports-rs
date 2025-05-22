use std::{collections::HashMap, error::Error as StdError};

use reqwest::StatusCode;

use crate::data::{
    Error, League, Matchup, Player, Roster, Sport, Transaction, TrendingPlayer, User,
};

const SLEEPER_API_BASE_URL: &str = "https://api.sleeper.app/v1";
const SLEEPER_CDN_BASE_URL: &str = "https://sleepercdn.com";

pub struct Client {
    base_url: String,
    client: reqwest::Client,
}
impl Client {
    pub fn new(base_url: Option<String>) -> Self {
        Client {
            base_url: base_url.unwrap_or(SLEEPER_API_BASE_URL.to_string()),
            client: reqwest::ClientBuilder::new()
                .build()
                .expect("dev error while building http client"),
        }
    }

    pub async fn fetch_all_players(
        &self,
        sport: &Sport,
    ) -> Result<HashMap<String, Player>, reqwest::Error> {
        match self
            .client
            .get(format!("{}/players/{}", self.base_url, sport))
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
        sport: &Sport,
        _type: String,
        lookback_hours: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<TrendingPlayer>, reqwest::Error> {
        let mut req = self.client.get(format!(
            "{}/players/{}/trending/{}",
            self.base_url, sport, _type
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

    pub async fn get_user(&self, user: String) -> Result<User, Error> {
        match self
            .client
            .get(format!("{}/user/{}", self.base_url, user.trim()))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<User>>().await {
                    Ok(opt_user) => match opt_user {
                        Some(user) => Ok(user),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_leagues_for_user(
        &self,
        user_id: &String,
        sport: &Sport,
        season: &String,
    ) -> Result<Vec<League>, Error> {
        match self
            .client
            .get(format!(
                "{}/user/{}/leagues/{}/{}",
                self.base_url,
                user_id.trim(),
                sport,
                season
            ))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<Vec<League>>>().await {
                    Ok(opt_league) => match opt_league {
                        Some(leagues) => Ok(leagues),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_league(&self, league_id: &String) -> Result<League, Error> {
        match self
            .client
            .get(format!("{}/league/{}", self.base_url, league_id.trim(),))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<League>>().await {
                    Ok(opt_league) => match opt_league {
                        Some(league) => Ok(league),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_rosters_for_league(&self, league_id: &String) -> Result<Vec<Roster>, Error> {
        match self
            .client
            .get(format!(
                "{}/league/{}/rosters",
                self.base_url,
                league_id.trim(),
            ))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<Vec<Roster>>>().await {
                    Ok(opt_rosters) => match opt_rosters {
                        Some(rosters) => Ok(rosters),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_users_for_league(&self, league_id: &String) -> Result<Vec<User>, Error> {
        let url = format!("{}/league/{}/users", self.base_url, league_id.trim(),);
        println!("GET {}", url);

        match self.client.get(url).send().await {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<Vec<User>>>().await {
                    Ok(opt_users) => match opt_users {
                        Some(users) => Ok(users),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_matchups(
        &self,
        league_id: &String,
        week: &String,
    ) -> Result<Vec<Matchup>, Error> {
        match self
            .client
            .get(format!(
                "{}/league/{}/matchups/{}",
                self.base_url,
                league_id.trim(),
                week,
            ))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<Vec<Matchup>>>().await {
                    Ok(opt_matchups) => match opt_matchups {
                        Some(matchups) => Ok(matchups),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }

    pub async fn get_transactions(
        &self,
        league_id: &String,
        week: &String,
    ) -> Result<Vec<Transaction>, Error> {
        match self
            .client
            .get(format!(
                "{}/league/{}/transactions/{}",
                self.base_url,
                league_id.trim(),
                week,
            ))
            .send()
            .await
        {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<Vec<Transaction>>>().await {
                    Ok(opt_transactions) => match opt_transactions {
                        Some(transactions) => Ok(transactions),
                        None => Err(Error::ResourceNotFound),
                    },
                    Err(json_error) => Err(Error::DecodeError(json_error)),
                },
                StatusCode::NOT_FOUND => Err(Error::ResourceNotFound),
                code => panic!("unknown error: {}", code),
            },
            Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => panic!("unknown error: {}", error),
        }
    }
}

pub fn avatar_url(avatar_id: String) -> String {
    format!("{}/avatars/{}", SLEEPER_CDN_BASE_URL, avatar_id)
}

pub fn avatar_thumbnail_url(avatar_id: String) -> String {
    format!("{}/avatars/thumbs/{}", SLEEPER_CDN_BASE_URL, avatar_id)
}

#[cfg(test)]
mod tests {
    use crate::Sport;

    use super::Client;
    use flate2::read::GzDecoder;
    use std::{
        collections::HashSet,
        fs::{self, File},
        io::{BufReader, Read},
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    const TEST_LEAGUE_ID: &str = "5054063865929450119";

    async fn setup() -> (MockServer, Client) {
        let mock_server = MockServer::start().await;
        let client = Client::new(Some(mock_server.uri()));
        (mock_server, client)
    }

    fn load_json_gz_file(filename: &str) -> std::io::Result<String> {
        let file = File::open(filename)?;
        let buf = BufReader::new(file);

        // Create a GzDecoder on top of the reader
        let mut decoder = GzDecoder::new(buf);

        // Read decompressed contents into a String
        let mut contents = String::new();
        decoder.read_to_string(&mut contents)?;
        Ok(contents)
    }

    #[tokio::test]
    async fn test_get_players_nfl() {
        let sport = Sport::NFL;
        let json_body = load_json_gz_file("testdata/players_nfl.json.gz").unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/players/{}", sport)))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_body)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let players = client.fetch_all_players(&sport).await.unwrap();
        assert_eq!(11360, players.len());
    }

    #[tokio::test]
    async fn test_get_matchups_for_league_week_1() {
        let week = 1;

        let json_body = fs::read_to_string("testdata/league_matchups_1.json").unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/league/{}/matchups/{}",
                TEST_LEAGUE_ID, week
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_body)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let matchups = client
            .get_matchups(&String::from(TEST_LEAGUE_ID), &week.to_string())
            .await
            .unwrap();
        assert_eq!(12, matchups.len());
        assert_eq!(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,]
                .iter()
                .map(|i| *i as u8)
                .collect::<HashSet<u8>>(),
            matchups
                .iter()
                .map(|matchup| matchup.roster_id.clone())
                .collect::<HashSet<u8>>()
        );
    }

    #[tokio::test]
    async fn test_get_rosters_for_league() {
        let json_body = fs::read_to_string("testdata/league_rosters.json").unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{}/rosters", TEST_LEAGUE_ID)))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_body)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let rosters = client
            .get_rosters_for_league(&String::from(TEST_LEAGUE_ID))
            .await
            .unwrap();
        assert_eq!(12, rosters.len());
        assert_eq!(
            vec![
                "388386020462698342",
                "359969312884605487",
                "220642397291036141",
                "211955574631057313",
                "890148532049792552",
                "7632227468501445432",
                "118247368628080948",
                "108802258691419802",
                "149110893312457277",
                "528873604862302116",
                "585263245137071266",
                "8600587490314352758",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>(),
            rosters
                .iter()
                .map(|roster| roster.owner_id.clone())
                .collect::<HashSet<String>>()
        );
    }

    #[tokio::test]
    async fn test_get_users_for_league() {
        let json_body = fs::read_to_string("testdata/league_users.json").unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{}/users", TEST_LEAGUE_ID)))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(json_body)
                    .insert_header("Content-Type", "application/json"),
            )
            .mount(&mock_server)
            .await;

        let users = client
            .get_users_for_league(&String::from(TEST_LEAGUE_ID))
            .await
            .unwrap();
        assert_eq!(12, users.len());
        assert_eq!(
            vec![
                "388386020462698342",
                "359969312884605487",
                "220642397291036141",
                "211955574631057313",
                "890148532049792552",
                "7632227468501445432",
                "118247368628080948",
                "108802258691419802",
                "149110893312457277",
                "528873604862302116",
                "585263245137071266",
                "8600587490314352758",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>(),
            users
                .iter()
                .map(|user| user.user_id.clone())
                .collect::<HashSet<String>>()
        );
    }
}
