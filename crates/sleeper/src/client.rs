use crate::{
    Draft, DraftPickMove, SportState,
    data::{
        Bracket, DraftPick, League, Matchup, Player, PlayoffBracketEntry, Roster, Sport,
        Transaction, TrendType, TrendingPlayer, User,
    },
};
use fantasy_sports_cache::Request;
use reqwest;
use std::{collections::HashMap, fmt::Display, path::PathBuf};

// const SLEEPER_API_BASE_URL_UNPUBLISHED: &str = "https://api.sleeper.app";
const SLEEPER_API_BASE_URL: &str = "https://api.sleeper.app/v1";
const SLEEPER_CDN_BASE_URL: &str = "https://sleepercdn.com";

pub struct Client {
    client: fantasy_sports_cache::Client,
}
impl Client {
    pub fn new() -> Self {
        Client {
            client: fantasy_sports_cache::Client::new(SLEEPER_API_BASE_URL.to_string())
                .with_http_client(
                    reqwest::ClientBuilder::new()
                        .gzip(true) // sets the accept-encoding: gzip and auto decodes the response body
                        // .default_headers(headers)
                        .build()
                        .expect("dev error while building http client"),
                ),
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

    pub fn request_count(&self) -> u64 {
        self.client.request_count()
    }

    /// Since rosters and draft picks contain Player IDs which look like
    /// "1042", "2403", "CAR", etc, you will need to know what those IDs map
    /// to. The /players call provides you the map necessary to look up any
    /// player.
    ///
    /// You should save this information on your own servers as this is not
    /// intended to be called every time you need to look up players due to the
    /// filesize being close to 5MB in size. You do not need to call this
    /// endpoint more than once per day.
    pub async fn fetch_all_players(
        &self,
        sport: &Sport,
    ) -> Result<HashMap<String, Player>, fantasy_sports_cache::Error> {
        self.fetch_all_players_request(sport).send().await
    }

    /// You can use this endpoint to get a list of trending players based on
    /// adds or drops in the past 24 hours.
    pub async fn get_trending_players(
        &self,
        sport: &Sport,
        r#type: TrendType,
        lookback_hours: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<TrendingPlayer>, fantasy_sports_cache::Error> {
        self.get_trending_players_request(sport, &r#type, &lookback_hours, &limit)
            .send()
            .await
    }

    /// Sleeper does not perform authentication as their API is read-only and
    /// only contains league information.
    ///
    /// Via the user resource, you can GET the user object by either providing
    /// the username or user_id of the user.
    pub async fn get_user<T: Id>(&self, user: T) -> Result<User, fantasy_sports_cache::Error> {
        let user: String = user.to_string();
        self.get_user_request(user.as_str()).send().await
    }

    /// Retreives all of the leagues that the user (provided numerical user_id)
    /// belongs to in a season.
    pub async fn get_leagues_for_user<T: Id>(
        &self,
        user_id: T,
        sport: &Sport,
        season: T,
    ) -> Result<Vec<League>, fantasy_sports_cache::Error> {
        let user_id = user_id.to_string();
        let season = season.to_string();
        self.get_leagues_for_user_request(user_id.as_str(), sport, season.as_str())
            .send()
            .await
    }

    /// This endpoint retrieves a specific league.
    pub async fn get_league<T: Id>(
        &self,
        league_id: T,
    ) -> Result<League, fantasy_sports_cache::Error> {
        let league_id: String = league_id.to_string();
        self.get_league_request(league_id.as_str()).send().await
    }

    /// This endpoint retrieves all rosters in a league.
    pub async fn get_rosters_in_league<T: Id>(
        &self,
        league_id: T,
    ) -> Result<Vec<Roster>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_rosters_in_league_request(league_id.as_str())
            .send()
            .await
    }

    /// This endpoint retrieves all users in a league.
    ///
    /// This also includes each user's display_name, avatar, and their metadata
    /// which sometimes includes a nickname they gave their team.
    pub async fn get_users_for_league<T: Id>(
        &self,
        league_id: T,
    ) -> Result<Vec<User>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_users_for_league_request(league_id.as_str())
            .send()
            .await
    }

    /// This endpoint retrieves all matchups in a league for a given week. Each
    /// object in the list represents one team. The two teams with the same
    /// matchup_id match up against each other.
    ///
    /// The starters is in an ordered list of player_ids, and players is a list
    /// of all player_ids in this matchup.
    ///
    /// The bench can be deduced by removing the starters from the players field.
    pub async fn get_matchups<T: Id>(
        &self,
        league_id: T,
        week: usize,
    ) -> Result<Vec<Matchup>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_matchups_request(league_id.as_str(), week)
            .send()
            .await
    }

    /// This endpoint retrieves all free agent transactions, waivers and trades
    pub async fn get_transactions<T: Id>(
        &self,
        league_id: T,
        week: usize,
    ) -> Result<Vec<Transaction>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_transactions_request(league_id.as_str(), week)
            .send()
            .await
    }

    /// This endpoint retrieves all traded picks in a league, including future
    /// picks.
    pub async fn get_traded_picks_in_league<T: Id>(
        &self,
        league_id: T,
    ) -> Result<Vec<DraftPickMove>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_traded_picks_in_league_request(league_id.as_str())
            .send()
            .await
    }

    /// This endpoint returns information about the current state for any sport.
    pub async fn get_sport_state(
        &self,
        sport: &Sport,
    ) -> Result<SportState, fantasy_sports_cache::Error> {
        self.get_sport_state_request(sport).send().await
    }

    /// This endpoint retrieves all drafts by a user.
    pub async fn get_drafts_for_user<T: Id>(
        &self,
        user_id: T,
        sport: &Sport,
        season: T,
    ) -> Result<Vec<Draft>, fantasy_sports_cache::Error> {
        let user_id = user_id.to_string();
        let season = season.to_string();
        self.get_drafts_for_user_request(user_id.as_str(), sport, season.as_str())
            .send()
            .await
    }

    /// This endpoint retrieves all drafts for a league. Keep in mind that a
    /// league can have multiple drafts, especially dynasty leagues.
    ///
    /// Drafts are sorted by most recent to earliest. Most leagues should only
    /// have one draft.
    pub async fn get_drafts_for_league<T: Id>(
        &self,
        league_id: T,
    ) -> Result<Vec<Draft>, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        self.get_drafts_for_league_request(league_id.as_str())
            .send()
            .await
    }

    /// This endpoint retrieves a specific draft.
    pub async fn get_draft<T: Id>(
        &self,
        draft_id: T,
    ) -> Result<Draft, fantasy_sports_cache::Error> {
        let draft_id = draft_id.to_string();
        self.get_draft_request(draft_id.as_str()).send().await
    }

    /// This endpoint retrieves all picks in a draft.
    pub async fn get_draft_picks<T: Id>(
        &self,
        draft_id: T,
    ) -> Result<Vec<DraftPick>, fantasy_sports_cache::Error> {
        let draft_id = draft_id.to_string();
        self.get_draft_picks_request(draft_id.as_str()).send().await
    }

    /// This endpoint retrieves the playoff bracket for a league for 4, 6, and
    /// 8 team playoffs.
    ///
    /// Each row represents a matchup between 2 teams.
    pub async fn get_playoff_bracket<T: Id>(
        &self,
        league_id: T,
        is_losers_bracket: bool,
    ) -> Result<Bracket, fantasy_sports_cache::Error> {
        let league_id = league_id.to_string();
        let bracket_matchups: Vec<PlayoffBracketEntry> = self
            .get_playoff_bracket_request(league_id.as_str(), is_losers_bracket)
            .send()
            .await?;
        Ok(Bracket {
            matchups: bracket_matchups,
            is_losers: is_losers_bracket,
        })
    }

    pub fn get_league_request<'a>(&'a self, league_id: &'a str) -> Request<'a, League> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn fetch_all_players_request<'a>(
        &'a self,
        sport: &'a Sport,
    ) -> Request<'a, HashMap<String, Player>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/players/{sport}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }
    pub fn get_trending_players_request<'a>(
        &'a self,
        sport: &'a Sport,
        r#type: &'a TrendType,
        lookback_hours: &'a Option<u32>, //TODO: implement this filter
        limit: &'a Option<u32>,
    ) -> Request<'a, Vec<TrendingPlayer>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/players/{sport}/trending/{}", r#type),
                filters: {
                    let filters_vec = [("lookback_hours", lookback_hours), ("limit", limit)]
                        .into_iter()
                        .filter_map(|(k, v)| match v {
                            Some(0) => None,
                            Some(value) => Some((k, value.to_string())),
                            None => None,
                        })
                        .collect::<Vec<(&str, String)>>();
                    if filters_vec.is_empty() {
                        None
                    } else {
                        Some(filters_vec)
                    }
                },
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_user_request<'a>(&'a self, user: &'a str) -> Request<'a, User> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/user/{user}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_leagues_for_user_request<'a>(
        &'a self,
        user_id: &'a str,
        sport: &'a Sport,
        season: &'a str,
    ) -> Request<'a, Vec<League>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/user/{user_id}/leagues/{sport}/{season}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_rosters_in_league_request<'a>(
        &'a self,
        league_id: &'a str,
    ) -> Request<'a, Vec<Roster>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/rosters"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_users_for_league_request<'a>(
        &'a self,
        league_id: &'a str,
    ) -> Request<'a, Vec<User>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/users"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_matchups_request<'a>(
        &'a self,
        league_id: &'a str,
        week: usize,
    ) -> Request<'a, Vec<Matchup>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/matchups/{week}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_transactions_request<'a>(
        &'a self,
        league_id: &'a str,
        week: usize,
    ) -> Request<'a, Vec<Transaction>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/transactions/{week}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_traded_picks_in_league_request<'a>(
        &'a self,
        league_id: &'a str,
    ) -> Request<'a, Vec<DraftPickMove>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/traded_picks"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_sport_state_request<'a>(&'a self, sport: &'a Sport) -> Request<'a, SportState> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/state/{sport}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_drafts_for_user_request<'a>(
        &'a self,
        user_id: &'a str,
        sport: &'a Sport,
        season: &'a str,
    ) -> Request<'a, Vec<Draft>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/user/{user_id}/drafts/{sport}/{season}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_drafts_for_league_request<'a>(
        &'a self,
        league_id: &'a str,
    ) -> Request<'a, Vec<Draft>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/league/{league_id}/drafts"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_draft_request<'a>(&'a self, draft_id: &'a str) -> Request<'a, Draft> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/draft/{draft_id}"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_draft_picks_request<'a>(&'a self, draft_id: &'a str) -> Request<'a, Vec<DraftPick>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!("/draft/{draft_id}/picks"),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    pub fn get_playoff_bracket_request<'a>(
        &'a self,
        league_id: &'a str,
        is_losers_bracket: bool,
    ) -> fantasy_sports_cache::Request<'a, Vec<PlayoffBracketEntry>> {
        self.client.build_request(
            fantasy_sports_cache::RequestContext {
                url: format!(
                    "/league/{league_id}/{bracket_name}",
                    bracket_name = if is_losers_bracket {
                        "losers_bracket"
                    } else {
                        "winners_bracket"
                    }
                ),
                filters: None,
            },
            false,
            Some(fantasy_sports_cache::ResponseBodyFormat::JSON),
        )
    }

    // user with basketball league: ManagerSilent58
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
    format!("{SLEEPER_CDN_BASE_URL}/avatars/{avatar_id}")
}

pub fn avatar_thumbnail_url(avatar_id: String) -> String {
    format!("{SLEEPER_CDN_BASE_URL}/avatars/thumbs/{avatar_id}")
}

#[cfg(test)]
mod client_tests {
    use crate::{SeasonType, Sport, SportState, TrendType, data::ScoringSettings};

    use super::Client;
    use chrono::NaiveDate;
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
        let client = Client::new().with_base_url(mock_server.uri());
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
            .get_matchups(String::from(TEST_LEAGUE_ID), week)
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
            .get_rosters_in_league(&String::from(TEST_LEAGUE_ID))
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
                .filter_map(|roster| roster.owner_id.clone())
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

    #[tokio::test]
    async fn test_get_drafts_for_nfl_league_w_orphaned_draft() {
        let league_id = "164974877262287888".to_string();
        let json_body =
            fs::read_to_string(format!("testdata/v1/league/{league_id}/drafts.json")).unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{league_id}/drafts")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let drafts = client.get_drafts_for_league(league_id).await.unwrap();
        assert_eq!(2, drafts.len());
        assert!(drafts[0].slot_to_roster_id.is_none()); // these shouldn't be here for the drafts_list
        assert!(drafts[1].slot_to_roster_id.is_none()); // these shouldn't be here for the drafts_list
    }

    #[tokio::test]
    async fn test_get_completed_nfl_draft() {
        let draft_id = "685822547406541191".to_string();
        let json_body = fs::read_to_string(format!("testdata/v1/draft/{draft_id}.json")).unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/draft/{draft_id}")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let draft = client.get_draft(draft_id).await.unwrap();
        assert!(draft.slot_to_roster_id.is_some());
        assert_eq!("2019", draft.season);
        assert_eq!(SeasonType::Regular, draft.season_type);
    }

    #[tokio::test]
    async fn test_get_pick_for_completed_nfl_draft() {
        let draft_id = "685822547406541191".to_string();
        let json_body =
            fs::read_to_string(format!("testdata/v1/draft/{draft_id}/picks.json")).unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/draft/{draft_id}/picks")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let draft_picks = client.get_draft_picks(draft_id).await.unwrap();
        assert_eq!(204, draft_picks.len());
        assert_eq!(35, draft_picks.iter().filter(|p| p.is_keeper).count()); // 35 keepers in this draft
    }

    #[tokio::test]
    async fn test_get_completed_playoff_bracket() {
        let league_id = "164974877262287888".to_string();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{league_id}/winners_bracket")))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    fs::read_to_string(format!(
                        "testdata/v1/league/{league_id}/winners_bracket.json"
                    ))
                    .expect("failed to load testdata file"),
                    "application/json",
                ),
            )
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{league_id}/losers_bracket")))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    fs::read_to_string(format!(
                        "testdata/v1/league/{league_id}/losers_bracket.json"
                    ))
                    .expect("failed to load testdata file"),
                    "application/json",
                ),
            )
            .mount(&mock_server)
            .await;
        let winners_bracket = client
            .get_playoff_bracket(league_id.clone(), false)
            .await
            .expect("failed to get winners bracket");
        debug_assert_eq!(
            Some(7),
            winners_bracket.champion(),
            "incorrect winner's bracket champion"
        );
        debug_assert_eq!(
            Some(9),
            winners_bracket.runner_up(),
            "incorrect winner's bracket runner-up"
        );

        // Losers bracket: the final is the "toilet bowl" between the two teams that kept
        // losing (matchup 7), so its *loser* is the champion. This is the behaviour of
        // Bracket::championship_match for losers brackets, which ignores Sleeper's own
        // placement ("p") field; see the FIXME on `impl Bracket`.
        let losers_bracket = client
            .get_playoff_bracket(league_id.clone(), true)
            .await
            .unwrap();
        debug_assert_eq!(
            Some(8),
            losers_bracket.champion(),
            "incorrect loser's bracket champion"
        );
        debug_assert_eq!(
            Some(11),
            losers_bracket.runner_up(),
            "incorrect loser's bracket runner-up"
        );
    }

    #[tokio::test]
    async fn test_traded_picks_for_league() {
        let league_id = "164974877262287888".to_string();
        let json_body =
            fs::read_to_string(format!("testdata/v1/league/{league_id}/traded_picks.json"))
                .unwrap();
        let (mock_server, client) = setup().await;
        Mock::given(method("GET"))
            .and(path(format!("/league/{league_id}/traded_picks")))
            .respond_with(ResponseTemplate::new(200).set_body_raw(json_body, "application/json"))
            .mount(&mock_server)
            .await;
        let traded_draft_picks = client.get_traded_picks_in_league(league_id).await.unwrap();
        assert_eq!(50, traded_draft_picks.len());
    }

    #[tokio::test]
    async fn test_sport_state() {
        let cases = [
            (
                Sport::NFL,
                SportState {
                    week: 1,
                    season_type: SeasonType::Regular,
                    season_start_date: NaiveDate::parse_from_str("2025-09-04", "%Y-%m-%d").unwrap(),
                    season: "2025".to_string(),
                    previous_season: "2024".to_string(),
                    leg: 1,
                    league_season: "2025".to_string(),
                    league_create_season: "2025".to_string(),
                    display_week: 1,
                    season_has_scores: true,
                },
            ),
            (
                Sport::NBA,
                SportState {
                    week: 0,
                    season_type: SeasonType::Off,
                    season_start_date: NaiveDate::parse_from_str("2025-10-21", "%Y-%m-%d").unwrap(),
                    season: "2025".to_string(),
                    previous_season: "2024".to_string(),
                    leg: 0,
                    league_season: "2025".to_string(),
                    league_create_season: "2025".to_string(),
                    display_week: 0,
                    season_has_scores: true,
                },
            ),
        ];
        let (mock_server, client) = setup().await;
        for (sport, expected_state) in cases {
            let json_body = fs::read_to_string(format!("testdata/v1/state/{sport}.json")).unwrap();
            Mock::given(method("GET"))
                .and(path(format!("/state/{sport}")))
                .respond_with(
                    ResponseTemplate::new(200).set_body_raw(json_body, "application/json"),
                )
                .mount(&mock_server)
                .await;
            let sport_state = client.get_sport_state(&sport).await.unwrap();
            debug_assert_eq!(expected_state, sport_state, "sport state does not match");
        }
    }
}

pub trait Id: Display {}

impl Id for usize {}
impl Id for &usize {}
impl Id for &str {}
impl Id for String {}
impl Id for &String {}
