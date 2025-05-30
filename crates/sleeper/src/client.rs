use std::collections::HashMap;

use reqwest::{Request, StatusCode};
use serde::de::DeserializeOwned;

use crate::data::{
    League, Matchup, Player, Roster, Sport, Transaction, TrendType, TrendingPlayer, User,
};
use thiserror;

// const SLEEPER_API_BASE_URL_UNPUBLISHED: &str = "https://api.sleeper.app";
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
                .gzip(true) // sets the accept-encoding: gzip and auto decodes the response body
                // .default_headers(headers)
                .build()
                .expect("dev error while building http client"),
        }
    }

    pub(crate) fn build_get_json_request(
        &self,
        path: String,
        filters: Option<Vec<(&str, String)>>,
    ) -> Result<Request, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);

        let mut request_builder = self.client.get(&url).header("Accept", "application/json");

        if let Some(filters) = filters {
            request_builder = request_builder.query(&filters)
        }
        request_builder.build()
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: String,
        filters: Option<Vec<(&str, String)>>,
    ) -> Result<T, Error> {
        let req = match self.build_get_json_request(path, filters) {
            Ok(req) => req,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "failed to build request: {:?}",
                    err
                )));
            }
        };
        let url = req.url().to_string();

        match self.client.execute(req).await {
            Ok(resp) => match resp.status() {
                StatusCode::OK => match resp.json::<Option<T>>().await {
                    Ok(opt_t) => match opt_t {
                        Some(t) => Ok(t),
                        None => Err(Error::NotFound(url)),
                    },
                    Err(err) => Err(Error::ResponseDeserializationError {
                        message: format!("{:?}", err),
                        url,
                    }),
                },
                StatusCode::NOT_FOUND => Err(Error::NotFound(url)),
                code => Err(Error::UnknownError(format!("unexpected status: {}", code))),
            },
            // Err(error) if error.is_connect() => Err(Error::ConnectError(error)),
            Err(error) => Err(Error::UnknownError(format!(
                "failed to send request: {:?}",
                error
            ))),
        }
    }

    pub async fn fetch_all_players(&self, sport: &Sport) -> Result<HashMap<String, Player>, Error> {
        self.get_json(format!("/players/{}", sport), None).await
    }

    pub async fn get_trending_players(
        &self,
        sport: &Sport,
        _type: TrendType,
        lookback_hours: Option<u32>, //TODO: implement this filter
        limit: Option<u32>,          //TODO: implement this filter
    ) -> Result<Vec<TrendingPlayer>, Error> {
        let filters_vec = [("lookback_hours", lookback_hours), ("limit", limit)]
            .into_iter()
            .filter_map(|(k, v)| match v {
                Some(0) => None,
                Some(value) => Some((k, value.to_string())),
                None => None,
            })
            .collect::<Vec<(&str, String)>>();
        let filters = if filters_vec.is_empty() {
            None
        } else {
            Some(filters_vec)
        };

        self.get_json(format!("/players/{}/trending/{}", sport, _type), filters)
            .await
    }

    pub async fn get_user(&self, user: String) -> Result<User, Error> {
        self.get_json(format!("/user/{}", user.trim()), None).await
    }

    pub async fn get_leagues_for_user(
        &self,
        user_id: &String,
        sport: &Sport,
        season: &String,
    ) -> Result<Vec<League>, Error> {
        self.get_json(
            format!("/user/{}/leagues/{}/{}", user_id.trim(), sport, season),
            None,
        )
        .await
    }

    pub async fn get_league(&self, league_id: &String) -> Result<League, Error> {
        self.get_json(format!("/league/{}", league_id.trim()), None)
            .await
    }

    pub async fn get_rosters_for_league(&self, league_id: &String) -> Result<Vec<Roster>, Error> {
        self.get_json(format!("/league/{}/rosters", league_id.trim()), None)
            .await
    }

    pub async fn get_users_for_league(&self, league_id: &String) -> Result<Vec<User>, Error> {
        self.get_json(format!("/league/{}/users", league_id.trim()), None)
            .await
    }

    pub async fn get_matchups(
        &self,
        league_id: String,
        week: String,
    ) -> Result<Vec<Matchup>, Error> {
        self.get_json(
            format!("/league/{}/matchups/{}", league_id.trim(), week),
            None,
        )
        .await
    }

    pub async fn get_transactions(
        &self,
        league_id: &String,
        week: &String,
    ) -> Result<Vec<Transaction>, Error> {
        self.get_json(
            format!("/league/{}/transactions/{}", league_id.trim(), week),
            None,
        )
        .await
    }

    // user with baseketball league: ManagerSilent58
    // https://api.sleeper.com/v1/user/267506301905934815/leagues/nba/2023
    // pub async fn get_stats(sport: Sport) {
    //     let season = "2022";
    //     let week = 18;

    //     let url = format!(
    //         "{SLEEPER_API_BASE_URL_UNPUBLISHED}/stats/{sport}/{season}/{week}?season_type=regular"
    //     );
    // }
    // pub async fn get_projections(sport: Sport) {
    //     let season = "2022";
    //     let week = 18;

    //     let url = format!(
    //         "{SLEEPER_API_BASE_URL_UNPUBLISHED}/projections/{sport}/{season}/{week}?season_type=regular"
    //     );
    // }
}

pub fn avatar_url(avatar_id: String) -> String {
    format!("{}/avatars/{}", SLEEPER_CDN_BASE_URL, avatar_id)
}

pub fn avatar_thumbnail_url(avatar_id: String) -> String {
    format!("{}/avatars/thumbs/{}", SLEEPER_CDN_BASE_URL, avatar_id)
}

#[cfg(test)]
mod tests {
    use crate::{Sport, TrendType, data::ScoringSettings};

    use super::Client;
    use flate2::read::GzDecoder;
    use std::{
        collections::{HashMap, HashSet},
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

    fn load_json_gz_file_raw(filename: &str) -> std::io::Result<Vec<u8>> {
        let file = File::open(filename)?;
        let mut buf_reader = BufReader::new(file);

        // Read decompressed contents into a String
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents)?;
        Ok(contents)
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
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;

        let players = client.fetch_all_players(&sport).await.unwrap();
        assert_eq!(11360, players.len());
    }

    #[tokio::test]
    async fn test_get_players_nfl_gzip() {
        let sport = Sport::NFL;
        let raw_compressed_json_body =
            load_json_gz_file_raw("testdata/players_nfl.json.gz").unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/players/{}", sport)))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(raw_compressed_json_body, "application/json")
                    .insert_header("Content-Encoding", "gzip"),
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
            .get_matchups(String::from(TEST_LEAGUE_ID), week.to_string())
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

    #[tokio::test]
    async fn test_get_trending_players_nfl() {
        let sport = Sport::NFL;
        let trending_type = TrendType::DROP;
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "/players/{}/trending/{}",
                sport, trending_type
            )))
            .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
            .mount(&mock_server)
            .await;
        let players = client
            .get_trending_players(&sport, trending_type, None, None)
            .await
            .unwrap();
        assert_eq!(0, players.len());

        let request_with_lookback = client
            .build_get_json_request(
                format!("/players/{}/trending/{}", sport, trending_type),
                Some(vec![("lookback_hours", "6".to_string())]),
            )
            .expect("should work fine");
        assert_eq!(
            format!(
                "{}/players/nfl/trending/drop?lookback_hours=6",
                mock_server.uri()
            ),
            request_with_lookback.url().to_string()
        );
        let request_with_limit = client
            .build_get_json_request(
                format!("/players/{}/trending/{}", sport, trending_type),
                Some(vec![("limit", "100".to_string())]),
            )
            .expect("should work fine");
        assert_eq!(
            format!("{}/players/nfl/trending/drop?limit=100", mock_server.uri()),
            request_with_limit.url().to_string()
        );
        let request_with_limit_and_lookback = client
            .build_get_json_request(
                format!("/players/{}/trending/{}", sport, trending_type),
                Some(vec![
                    ("lookback_hours", "10".to_string()),
                    ("limit", "100".to_string()),
                ]),
            )
            .expect("should work fine");
        assert_eq!(
            format!(
                "{}/players/nfl/trending/drop?lookback_hours=10&limit=100",
                mock_server.uri()
            ),
            request_with_limit_and_lookback.url().to_string()
        );
    }

    #[tokio::test]
    async fn test_get_nba_2023_leagues_for_user() {
        let user_id = "267506301905934815".to_string();
        let sport = Sport::NBA;
        let season = 2023.to_string();
        let json_body = fs::read_to_string(format!(
            "testdata/v1/user/{user_id}/leagues/{sport}/{season}.json"
        ))
        .unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/user/{user_id}/leagues/{sport}/{season}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let nba_leagues = client
            .get_leagues_for_user(&user_id, &sport, &season)
            .await
            .unwrap();
        assert_eq!(2, nba_leagues.len());
        assert_eq!(
            ScoringSettings::Basketball {
                ast: 1.0,
                blk: 3.0,
                bonus_ast_15p: 2.0,
                bonus_pt_40p: 1.0,
                bonus_pt_50p: 2.0,
                bonus_reb_20p: 2.0,
                dd: 1.0,
                ff: -2.0,
                fgmi: -0.5,
                ftmi: -0.5,
                pts: 1.0,
                reb: 1.0,
                stl: 3.0,
                td: 2.0,
                tf: -1.0,
                to: -1.0,
                tpm: 0.5,
                other: HashMap::new(),
            },
            nba_leagues[0].scoring_settings
        );
        assert_eq!(
            ScoringSettings::Basketball {
                ast: 1.0,
                blk: 3.0,
                bonus_ast_15p: 2.0,
                bonus_pt_40p: 0.0,
                bonus_pt_50p: 2.0,
                bonus_reb_20p: 2.0,
                dd: 1.0,
                ff: -2.0,
                fgmi: -0.5,
                ftmi: -0.5,
                pts: 1.0,
                reb: 1.0,
                stl: 3.0,
                td: 2.0,
                tf: -1.0,
                to: -1.0,
                tpm: 0.5,
                other: HashMap::new(),
            },
            nba_leagues[1].scoring_settings
        );
        assert!(!nba_leagues[0].scoring_settings.has_other_settings());
        assert!(!nba_leagues[1].scoring_settings.has_other_settings());
    }

    #[tokio::test]
    async fn test_get_nfl_2024_leagues_for_user() {
        let user_id = "267506301905934815".to_string();
        let sport = Sport::NFL;
        let season = 2024.to_string();
        let json_body = fs::read_to_string(format!(
            "testdata/v1/user/{user_id}/leagues/{sport}/{season}.json"
        ))
        .unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/user/{user_id}/leagues/{sport}/{season}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let nfl_leagues = client
            .get_leagues_for_user(&user_id, &sport, &season)
            .await
            .unwrap();
        assert_eq!(6, nfl_leagues.len());
        assert_eq!(
            ScoringSettings::Football {
                sack: 1.0,
                fgm_40_49: 4.0,
                pass_int: -1.0,
                pts_allow_0: 10.0,
                pass_2pt: 2.0,
                st_td: 6.0,
                rec_td: 6.0,
                fgm_30_39: 3.0,
                xpmiss: -1.0,
                rush_td: 6.0,
                rec_2pt: 2.0,
                st_fum_rec: 1.0,
                fgmiss: -1.0,
                ff: 1.0,
                rec: 1.0,
                pts_allow_14_20: 1.0,
                fgm_0_19: 3.0,
                int: 2.0,
                def_st_fum_rec: 1.0,
                fum_lost: -2.0,
                pts_allow_1_6: 7.0,
                fgm_20_29: 3.0,
                pts_allow_21_27: 0.0,
                xpm: 1.0,
                rush_2pt: 2.0,
                fum_rec: 2.0,
                def_st_td: 6.0,
                fgm_50p: 5.0,
                def_td: 6.0,
                safe: 2.0,
                pass_yd: 0.04,
                blk_kick: 2.0,
                pass_td: 4.0,
                rush_yd: 0.1,
                fum: 0.0,
                pts_allow_28_34: -1.0,
                pts_allow_35p: -4.0,
                fum_rec_td: 6.0,
                rec_yd: 0.1,
                def_st_ff: 1.0,
                pts_allow_7_13: 4.0,
                st_ff: 1.0,
                def_kr_td: 0.0,
                kr_td: 0.0,
                def_pr_td: 0.0,
                pr_td: 0.0,
                other: HashMap::new(),
                bonus_rec_yd_200: 0.0,
                pass_td_40p: 0.0,
                bonus_rush_yd_200: 0.0,
                bonus_pass_yd_400: 0.0,
                rush_td_40p: 0.0,
                bonus_rec_te: 0.0,
                pass_int_td: 0.0,
                pass_cmp_40p: 0.0,
                rec_td_40p: 0.0,
                rush_40p: 0.0,
                rec_40p: 0.0,
            },
            nfl_leagues[0].scoring_settings
        );
        assert!(!nfl_leagues[0].scoring_settings.has_other_settings());
        assert!(!nfl_leagues[1].scoring_settings.has_other_settings());
        assert!(!nfl_leagues[2].scoring_settings.has_other_settings());
        assert!(!nfl_leagues[3].scoring_settings.has_other_settings());
        assert!(!nfl_leagues[4].scoring_settings.has_other_settings());
        assert!(!nfl_leagues[5].scoring_settings.has_other_settings());
    }

    #[tokio::test]
    async fn test_get_nfl_2018_leagues_for_user() {
        let user_id = "727598467304811923".to_string();
        let sport = Sport::NFL;
        let season = 2018.to_string();
        let json_body = fs::read_to_string(format!(
            "testdata/v1/user/{user_id}/leagues/{sport}/{season}.json"
        ))
        .unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/user/{user_id}/leagues/{sport}/{season}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let nfl_leagues = client
            .get_leagues_for_user(&user_id, &sport, &season)
            .await
            .unwrap();
        assert_eq!(1, nfl_leagues.len());
        assert_eq!(
            ScoringSettings::Football {
                sack: 1.0,
                fgm_40_49: 4.0,
                pass_int: -1.0,
                pts_allow_0: 10.0,
                pass_2pt: 2.0,
                st_td: 6.0,
                rec_td: 6.0,
                fgm_30_39: 3.0,
                xpmiss: -1.0,
                rush_td: 6.0,
                rec_2pt: 2.0,
                st_fum_rec: 1.0,
                fgmiss: 0.0,
                ff: 1.0,
                rec: 1.0,
                pts_allow_14_20: 1.0,
                fgm_0_19: 3.0,
                int: 2.0,
                def_st_fum_rec: 0.0,
                fum_lost: -2.0,
                pts_allow_1_6: 7.0,
                fgm_20_29: 3.0,
                pts_allow_21_27: 0.0,
                xpm: 1.0,
                rush_2pt: 2.0,
                fum_rec: 2.0,
                def_st_td: 0.0,
                fgm_50p: 5.0,
                def_td: 6.0,
                safe: 2.0,
                pass_yd: 0.03999999910593033,
                blk_kick: 2.0,
                pass_td: 4.0,
                rush_yd: 0.10000000149011612,
                fum: 0.0,
                pts_allow_28_34: -1.0,
                pts_allow_35p: -4.0,
                rec_yd: 0.10000000149011612,
                def_st_ff: 0.0,
                pts_allow_7_13: 4.0,
                st_ff: 1.0,
                def_kr_td: 0.0,
                fum_rec_td: 0.0,
                pass_int_td: 0.0,
                kr_td: 0.0,
                def_pr_td: 0.0,
                pr_td: 0.0,
                bonus_rec_yd_200: 0.0,
                pass_td_40p: 0.0,
                bonus_rush_yd_200: 0.0,
                bonus_pass_yd_400: 0.0,
                rush_td_40p: 0.0,
                bonus_rec_te: 0.0,
                pass_cmp_40p: 0.0,
                rec_td_40p: 0.0,
                rush_40p: 0.0,
                rec_40p: 0.0,
                other: HashMap::new(),
            },
            nfl_leagues[0].scoring_settings
        );
        assert!(!nfl_leagues[0].scoring_settings.has_other_settings());
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    // #[error("Request error: {0}")]
    // RequestError(#[from] reqwest::Error),
    #[error("Response Deserialization error: {message} url: {url}")]
    ResponseDeserializationError { message: String, url: String },
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("unknown error: {0}")]
    UnknownError(String),
}
