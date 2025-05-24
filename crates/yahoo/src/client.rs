use crate::data::{
    FantasyContent, FantasyContentWrapper, Game, GameCode, GameKey, League, LeagueKey, Player,
    Roster, Scoreboard, Settings, Standings, Team, TeamKey, Token, Transaction, User,
};
use chrono::NaiveDate;
use core::str;
use itertools::join;
use quick_xml::de::from_str;

const USER_AGENT: &str = "reqwest";

const YAHOO_API_AUTH_BASE_URL: &str = "https://api.login.yahoo.com/oauth2";
const YAHOO_FANTASY_API_BASE_URL: &str = "https://fantasysports.yahooapis.com/fantasy/v2";

pub struct Client {
    base_url: String,
    client: reqwest::Client,
    client_no_redirects: reqwest::Client,
    client_id: String,
    redirect_uri: String,
    code_verifier: Option<String>,
    pub token: Option<Token>,
}
impl Client {
    pub fn with_token(
        client_id: String,
        redirect_uri: String,
        token: Token,
        base_url: Option<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap();
        let client_no_redirects = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        Client {
            base_url: base_url.unwrap_or(YAHOO_FANTASY_API_BASE_URL.to_string()),
            client,
            client_no_redirects,
            client_id,
            redirect_uri,
            code_verifier: Option::None,
            token: Some(token),
        }
    }

    pub fn new(client_id: String, redirect_uri: String, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("reqwest")
            .build()
            .unwrap();
        let client_no_redirects = reqwest::Client::builder()
            .user_agent("reqwest")
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        Client {
            base_url: base_url.unwrap_or(YAHOO_FANTASY_API_BASE_URL.to_string()),
            client,
            client_no_redirects,
            client_id,
            redirect_uri,
            code_verifier: Option::None,
            token: Option::None,
        }
    }

    pub async fn get_request_auth_url(&mut self) -> Result<String, reqwest::Error> {
        let code_verifier = pkce::code_verifier(43);
        let code_challenge = pkce::code_challenge(&code_verifier);
        let code_challenge_method = "S256";

        let req = self
            .client_no_redirects
            .get(format!("{}/request_auth", YAHOO_API_AUTH_BASE_URL).as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("response_type", "code"),
                ("language", "en-us"),
                ("code_challenge", &code_challenge),
                ("code_challenge_method", code_challenge_method),
            ])
            .build()?;
        let resp = self.client_no_redirects.execute(req).await?;
        if resp.status().is_redirection() {
            println!("Success!");
        } else {
            println!("Failed: {}", resp.status());
        }

        // println!("Response: {:#?}", resp);

        let location = resp.headers().get("location").unwrap();
        // println!("Location: {}", location.to_str().unwrap());

        self.code_verifier = Some(String::from_utf8(code_verifier).unwrap());

        return Ok(location.to_str().unwrap().to_string());
    }

    pub async fn get_token(&mut self, code: &str) -> Result<Token, reqwest::Error> {
        let code_verifier = self.code_verifier.as_ref().unwrap();

        let form = [
            ("client_id", self.client_id.as_str()),
            ("code", code),
            ("code_verifier", code_verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "oob"),
        ];
        let req = self
            .client
            .post(format!("{}/get_token", YAHOO_API_AUTH_BASE_URL).as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&form)
            .build()?;
        let resp = self.client.execute(req).await?;
        if resp.status().is_success() {
            println!("Success!");
        } else {
            println!("Failed: {}", resp.status());
        }

        // println!("Response: {:#?}", resp);

        // println!("Status: {}", resp.status());

        let body = resp.json::<Token>().await?;
        self.token = Some(body.clone());
        return Ok(body);
    }

    pub async fn refresh_token(&mut self) -> Result<Token, reqwest::Error> {
        let form = [
            ("client_id", self.client_id.as_str()),
            (
                "refresh_token",
                self.token.as_mut().unwrap().refresh_token.as_str(),
            ),
            ("grant_type", "refresh_token"),
            ("redirect_uri", "oob"),
        ];
        let req = self
            .client
            .post(format!("{}/get_token", YAHOO_API_AUTH_BASE_URL).as_str())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&form)
            .build()?;
        let resp = self.client.execute(req).await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap();
            println!("Err: body: {}", body_text);
            panic!("Failed to refresh token: {}", status);
        }

        let body = resp.json::<Token>().await?;
        self.token = Some(body.clone());
        return Ok(body);
    }

    pub(crate) async fn get(&self, url: &str) -> Result<FantasyContentWrapper, reqwest::Error> {
        println!("GET {}", url);

        let req = self
            .client
            .get(url)
            .header("Accept", "application/xml")
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap().access_token),
            )
            .build()?;
        let resp = self.client.execute(req).await?;

        if resp.status().is_client_error() {
            let code = resp.status().as_u16();
            let body = resp.text().await.expect("couldn't read response body");
            println!("Status:   {}", code);
            println!("Raw Body: {}", body);
            match serde_xml_rs::from_str::<crate::data::Error>(body.as_str()) {
                Ok(yahoo_error) => {
                    // OAuth oauth_problem="token_expired", realm="yahooapis.com"
                    panic!("Got client error: {}", yahoo_error.description)
                }
                Err(error) => panic!("Got unknown error: {}", error),
            }
        } else if resp.status().is_server_error() {
            let code = resp.status().as_u16();
            let body = resp.text().await.expect("couldn't read response body");
            panic!("Got server error ({}): {}", code, body)
        }

        let body_text = resp.text().await?;
        println!("Body: {}", body_text);

        return match from_str(body_text.as_str()) {
            Ok(content) => Ok(content),
            Err(error) => {
                println!("{:#?}", error);
                panic!("unable to deserialize xml: {}", error)
            }
        };
    }

    /// This fetches the scorebaord for the league. It defaults to the final week or day.
    ///
    /// use date to filter in NBA, NHL league
    /// use week to filter in NFL league
    pub async fn get_league_scoreboard(
        &self,
        league_key: &LeagueKey,
        week: Option<i32>,
        date: Option<NaiveDate>,
    ) -> Result<Scoreboard, reqwest::Error> {
        let url = format!(
            "{}/league/{}/scoreboard{}",
            self.base_url,
            league_key,
            if let Some(week) = week {
                format!(";week={}", week)
            } else if let Some(date) = date {
                format!(";date={}", date)
            } else {
                String::from("")
            }
        );

        return match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.scoreboard {
                    Some(scoreboard) => Ok(scoreboard),
                    None => panic!(),
                },
                _ => panic!(),
            },
            Err(error) => panic!("{}", error),
        };
    }

    pub async fn get_league_settings(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Settings, reqwest::Error> {
        let url = format!("{}/league/{}/settings", self.base_url, league_key);

        return match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.settings {
                    Some(settings) => Ok(settings),
                    None => panic!(),
                },
                _ => panic!(),
            },
            Err(error) => panic!("failed to fetch league settings content: {}", error),
        };
    }

    pub async fn get_league_standings(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Standings, reqwest::Error> {
        let url = format!("{}/league/{}/standings", self.base_url, league_key);

        return match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.standings {
                    Some(standings) => Ok(standings),
                    None => panic!("failed to fetch league standings: missing"),
                },
                _ => panic!("failed to fetch league standings content: invalid content variant: "),
            },
            Err(error) => panic!("failed to fetch league standings content: {}", error),
        };
    }

    pub async fn get_league_transactions(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Vec<Transaction>, reqwest::Error> {
        let url = format!("{}/league/{}/transactions", self.base_url, league_key);

        return match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.transactions {
                    Some(transactions) => Ok(transactions.transactions),
                    None => Ok(vec![]),
                },
                _ => {
                    panic!("failed to fetch league transactions content: invalid content variant: ")
                }
            },
            Err(error) => panic!("failed to fetch league transactions content: {}", error),
        };
    }

    pub async fn get_game(&self, game_key: GameKey) -> Result<Game, reqwest::Error> {
        let url = format!("{}/game/{}", self.base_url, game_key.to_string());
        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Game(game) => return Ok(game),
                _ => panic!("failed to fetch game content: invalid content variant: "),
            },
            Err(error) => panic!("failed to fetch game content: {}", error),
        }
    }

    pub async fn get_games(
        &self,
        game_codes: Vec<GameCode>,
        seasons: Vec<&str>,
    ) -> Result<Vec<Game>, reqwest::Error> {
        let mut filter = String::new();

        if !game_codes.is_empty() {
            filter.push_str(";game_codes=");
            filter.push_str(join(game_codes, ",").as_str());
        }
        if !seasons.is_empty() {
            filter.push_str(";seasons=");
            filter.push_str(seasons.join(",").as_str());
        }

        let url = format!("{}/games{}", self.base_url, filter);

        println!("GET {}", url);

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Games(games) => return Ok(games.games),
                _ => panic!("failed to fetch users content: invalid content variant: "),
            },
            Err(error) => panic!("failed to fetch games content: {}", error),
        }
    }

    pub async fn get_users(&self, use_login: bool) -> Result<Vec<User>, reqwest::Error> {
        let mut query = "";
        if use_login {
            query = ";use_login=1";
        }

        let url = format!("{}/users{}", self.base_url, query);

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Users(users) => return Ok(users.users),
                _ => panic!("failed to fetch users content: invalid content variant: "),
            },
            Err(error) => panic!("failed to fetch users content: {}", error),
        }
    }

    /// league_key is {game_code}.l.{league_id}
    ///
    /// # Panics
    ///
    /// Panics if .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn get_league(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Option<League>, reqwest::Error> {
        let url = format!("{}/league/{}", self.base_url, league_key.to_string());

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => return Ok(Some(league)),
                _ => panic!("failed to fetch league content: invalid content variant: "),
            },
            Err(error) => panic!("failed to fetch league content: {}", error),
        }
    }

    pub async fn get_teams_for_user(&self, game_key: GameKey) -> Result<Vec<Team>, reqwest::Error> {
        let url = format!(
            "{}/users;use_login=1/games;game_keys={}/teams",
            self.base_url,
            game_key.to_string()
        );

        let content = self.get(url.as_str()).await.unwrap();

        let mut return_me: Vec<Team> = vec![];

        if let FantasyContent::Users(users) = content.content {
            for user in users.users {
                if let Some(games) = user.games {
                    for game in games.games {
                        if let Some(teams) = game.teams {
                            for team in teams.teams {
                                return_me.push(team);
                            }
                        }
                    }
                }
            }
        }

        Ok(return_me)
    }

    pub async fn get_teams_in_league(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Vec<Team>, reqwest::Error> {
        let url = format!("{}/league/{}/teams", self.base_url, league_key,);

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.teams {
                    Some(teams) => Ok(teams.teams),
                    None => Ok(vec![]),
                },
                _ => panic!("unsupported thinggggasdfasdf"),
            },
            Err(_) => todo!(),
        }
    }

    pub async fn get_players_for_league(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Vec<Player>, reqwest::Error> {
        let url = format!("{}/league/{}/players", self.base_url, league_key,);

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.players {
                    Some(players) => Ok(players.players),
                    None => Ok(vec![]),
                },
                _ => panic!("got another fantasy content variant than expected"),
            },
            Err(error) => panic!("failed to fetch player in league {}: {}", league_key, error),
        }
    }

    pub async fn get_players_for_leagues(
        &self,
        league_keys: Vec<&LeagueKey>,
    ) -> Result<Vec<League>, reqwest::Error> {
        let league_keys_str = join(league_keys, ",");

        let url = format!(
            "{}/leagues;league_keys={}/players",
            self.base_url, league_keys_str,
        );

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Leagues(leagues) => Ok(leagues.leagues),
                _ => panic!("got another fantasy content variant than expected"),
            },
            Err(error) => panic!(
                "failed to fetch players in leagues {}: {}",
                league_keys_str, error
            ),
        }
    }

    /// Documentation: https://developer.yahoo.com/fantasysports/guide/#roster-resource
    ///
    /// For NFL, you can specify a week parameter to retrieve a specific week – otherwise it will default to the current roster
    /// https://fantasysports.yahooapis.com/fantasy/v2/team//roster;week=10
    ///
    /// For MLB, NHL, or NBA, you can specify a date parameter to retrieve a specific date – otherwise it will default to today’s roster.
    /// https://fantasysports.yahooapis.com/fantasy/v2/team//roster;date=2011-05-01
    pub async fn get_roster_for_team(&self, team_key: &TeamKey) -> Result<Roster, reqwest::Error> {
        let url = format!("{}/team/{}/roster", self.base_url, team_key,);

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Team(team) => match team.roster {
                    Some(roster) => Ok(roster),
                    None => panic!("asdfasdfasdfasdfffffff"),
                },
                _ => panic!("got another fantasy content variant than expected"),
            },
            Err(error) => panic!("failed to fetch roster for team {}: {}", team_key, error),
        }
    }

    pub async fn get_stats_for_team(
        &self,
        team_key: &TeamKey,
        week: Option<i32>,
        date: Option<NaiveDate>,
    ) -> Result<Roster, reqwest::Error> {
        let url = format!(
            "{}/team/{}/stats{}",
            self.base_url,
            team_key,
            if let Some(week) = week {
                format!(":type=week;week={}", week)
            } else if let Some(date) = date {
                format!(":type=date;date={}", date)
            } else {
                String::from("")
            }
        );

        match self.get(url.as_str()).await {
            Ok(content) => match content.content {
                FantasyContent::Team(team) => match team.roster {
                    Some(roster) => Ok(roster),
                    None => panic!("asdfasdfasdfasdfffffff"),
                },
                _ => panic!("got another fantasy content variant than expected"),
            },
            Err(error) => panic!("failed to fetch roster for team {}: {}", team_key, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{Client, GameCode, GameKey, LeagueKey, Token, data::DraftStatus};
    use chrono::NaiveDate;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    async fn setup_with_200(p: &str) -> (Client, MockServer) {
        let mock_server = wiremock::MockServer::builder().start().await;
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(
                        fs::read_to_string(format!("tests/data/fantasy/v2{}.xml", p)).unwrap(),
                    )
                    .insert_header("Content-Type", "application/xml"),
                // < date: Sat, 24 May 2025 03:40:09 GMT
                // < p3p: policyref="https://policies.yahoo.com/w3c/p3p.xml", CP="CAO DSP COR CUR ADM DEV TAI PSA PSD IVAi IVDi CONi TELo OTPi OUR DELi SAMi OTRi UNRi PUBi IND PHY ONL UNI PUR FIN COM NAV INT DEM CNT STA POL HEA PRE LOC GOV"
                // < x-yahoo-request-id: 07pu1ftk32fsp
                // < x-yahoo-host: fantasy-sports-api--public-production-bf1-7dbb5fc5b6-4tqfj
                // < x-frame-options: SAMEORIGIN
                // < vary: Origin,Accept-Encoding
                // < set-cookie: SPTB=d=tYJxGqs8o7ItEP3yKz.Sq_ZumH4_fU_hbwnv3zGMBxqOoti45lmLsWkck6obDC7Xz5chDIx1HsP.O9LWhI6DRTy.w.8e6k3sKt0X3yZgf4g-&v=1; expires=Mon, 23-Jun-2025 03:40:09 GMT; Max-Age=2592000; path=/; domain=fantasysports.yahooapis.com
                // < cache-control: private,max-age=0,stale-while-revalidate=0,stale-if-error=0
                // < content-length: 881
                // < content-type: application/xml; charset=utf-8
                // < x-envoy-upstream-service-time: 33
                // < server: ATS
                // < age: 0
                // < strict-transport-security: max-age=31536000
                // < referrer-policy: strict-origin-when-cross-origin
                // < x-content-type-options: nosniff
                // < x-xss-protection: 1; mode=block
            )
            .mount(&mock_server)
            .await;

        let fixture = Client::with_token(
            "".to_string(),
            "".to_string(),
            Token {
                access_token: "foo".to_string(),
                token_type: "bearer".to_string(),
                expires_in: 3600,
                refresh_token: "bar".to_string(),
                scope: None,
            },
            Some(mock_server.uri()),
        );
        (fixture, mock_server)
    }

    #[tokio::test]
    async fn test_get_games_2019_nfl_happy() {
        let (fixture, _mock_server) = setup_with_200("/games;game_codes=nfl;seasons=2019").await;

        let games = fixture
            .get_games(vec![GameCode::NFL], vec!["2019"])
            .await
            .expect("should pass");

        assert_eq!(1, games.len());
        assert_eq!(GameCode::NFL, games[0].code);
        assert_eq!(390, games[0].game_id);
        assert_eq!(390, games[0].game_key);
        assert_eq!("Football", games[0].name);
        assert_eq!("2019", games[0].season);
        assert_eq!(None, games[0].teams);
    }

    #[tokio::test]
    async fn test_get_games_2019_poo_none_found() {
        let (fixture, _mock_server) = setup_with_200("/games;game_codes=poo;seasons=2019").await;
        let games = fixture
            .get_games(vec![GameCode::OTHER("poo".to_string())], vec!["2019"])
            .await
            .expect("should return ok (although should be empty)");

        assert_eq!(0, games.len());
    }

    #[tokio::test]
    async fn test_get_games_2009_2025_nfl() {
        let (fixture, _mock_server) = setup_with_200("/games;game_codes=nfl;seasons=2009,2010,2011,2012,2013,2014,2015,2016,2017,2018,2019,2020,2021,2022,2023,2024,2025").await;
        let games = fixture
            .get_games(
                vec![GameCode::NFL],
                vec![
                    "2009", "2010", "2011", "2012", "2013", "2014", "2015", "2016", "2017", "2018",
                    "2019", "2020", "2021", "2022", "2023", "2024", "2025",
                ],
            )
            .await
            .expect("should return ok (although should be empty)");

        assert_eq!(17, games.len());
        // all are NFL
        for game in &games {
            assert_eq!(GameCode::NFL, game.code);
            assert_eq!("Football", game.name);
        }
    }

    #[tokio::test]
    async fn test_get_teams() {
        let (fixture, _mock_server) =
            setup_with_200("/users;use_login=1/games;game_keys=390/teams").await;
        let teams = fixture
            .get_teams_for_user(
                GameKey::Id(390), // 2019 nfl
            )
            .await
            .expect("should return ok");

        assert_eq!(1, teams.len());

        assert_eq!("390.l.299227.t.1", teams[0].team_key);
        assert_eq!(1, teams[0].team_id);
        assert_eq!("Stormy Badgers 01", teams[0].name);
        assert_eq!(None, teams[0].clinched_playoffs);
        assert_eq!(Some(1), teams[0].division_id);
        assert_eq!(Some(100), teams[0].faab_balance);
        assert_eq!(false, teams[0].has_draft_grade);
        assert_eq!(Some(true), teams[0].is_owned_by_current_login);
        assert_eq!("head", teams[0].league_scoring_type);
        assert_eq!(0, teams[0].number_of_moves);
        assert_eq!(0, teams[0].number_of_trades);
        assert_eq!(1, teams[0].team_logos().len());
        assert_eq!("large", teams[0].team_logos()[0].size);
        assert_eq!(
            "https://yahoofantasysports-res.cloudinary.com/image/upload/t_s192sq/fantasy-logos/30368783128_2bdfc8.jpg",
            teams[0].team_logos()[0].url
        );
        assert_eq!(1, teams[0].managers().len());
        assert_eq!(Some(true), teams[0].managers()[0].is_commissioner);
        assert_eq!(Some(true), teams[0].managers()[0].is_current_login);
        assert_eq!(
            Some("https://example.invalid/ade39e2a327f69f0.png".to_string()),
            teams[0].managers()[0].image_url
        );
        assert_eq!(Some(580), teams[0].managers()[0].felo_score);
        assert_eq!(
            Some(crate::data::FeloTier::Bronze),
            teams[0].managers()[0].felo_tier
        );
        assert_eq!(None, teams[0].waiver_priority); // tests that <waiver_priority/> is serialized to None
    }

    #[tokio::test]
    async fn test_get_league_complete() {
        let league_key = LeagueKey::new(GameKey::Id(380).to_string().as_str(), 129397);
        let (fixture, _mock_server) = setup_with_200("/league/380.l.129397").await;
        let opt_league = fixture
            .get_league(
                &league_key, // 2018 nfl
            )
            .await
            .expect("should return ok");

        let league = opt_league.expect("should be there");
        assert_eq!(league_key, league.league_key);
        assert_eq!(Some("371_20028".to_string()), league.renew);
        assert_eq!("private", league.league_type);
        assert_eq!(false, league.is_pro_league);
        assert_eq!(false, league.is_cash_league);
        assert_eq!(false, league.is_plus_league);
        assert_eq!(
            NaiveDate::from_ymd_opt(2018, 9, 6).unwrap(),
            league.start_date
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2018, 12, 24).unwrap(),
            league.end_date
        );
        assert_eq!(DraftStatus::PostDraft, league.draft_status); // here it is
        assert_eq!(Some(true), league.is_finished); // here it is
    }

    #[tokio::test]
    async fn test_get_league_incomplete() {
        let (fixture, _mock_server) = setup_with_200("/league/390.l.299227").await;
        let opt_league = fixture
            .get_league(
                &LeagueKey::new(GameKey::Id(390).to_string().as_str(), 299227), // 2019 nfl
            )
            .await
            .expect("should return ok");

        let league = opt_league.expect("should be there");
        assert_eq!(
            LeagueKey::new(GameKey::Id(390).to_string().as_str(), 299227),
            league.league_key
        );
        assert_eq!(Some("380_129397".to_string()), league.renew);
        assert_eq!("private", league.league_type);
        assert_eq!(false, league.is_pro_league);
        assert_eq!(false, league.is_cash_league);
        assert_eq!(false, league.is_plus_league);
        assert_eq!(
            NaiveDate::from_ymd_opt(2019, 9, 5).unwrap(),
            league.start_date
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2019, 12, 23).unwrap(),
            league.end_date
        );
        assert_eq!(DraftStatus::PreDraft, league.draft_status); // here it is
        assert_eq!(None, league.is_finished); // here it is
    }
}
