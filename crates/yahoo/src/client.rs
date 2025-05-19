use crate::data::{
    FantasyContent, FantasyContentWrapper, Game, GameCode, GameKey, League, LeagueKey, Player,
    Roster, Scoreboard, Settings, Standings, Team, TeamKey, Token, Transaction, User,
};
use chrono::NaiveDate;
use core::str;
use itertools::join;
use quick_xml::de::from_str;

const YAHOO_API_AUTH_BASE_URL: &str = "https://api.login.yahoo.com/oauth2";
const YAHOO_FANTASY_API_BASE_URL: &str = "https://fantasysports.yahooapis.com/fantasy/v2";

pub struct Client {
    client: reqwest::Client,
    client_no_redirects: reqwest::Client,
    client_id: String,
    redirect_uri: String,
    code_verifier: Option<String>,
    pub token: Option<Token>,
}
impl Client {
    pub fn new(client_id: String, redirect_uri: String) -> Self {
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

    pub async fn get(&self, url: &str) -> Result<FantasyContentWrapper, reqwest::Error> {
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
            YAHOO_FANTASY_API_BASE_URL,
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
        let url = format!(
            "{}/league/{}/settings",
            YAHOO_FANTASY_API_BASE_URL, league_key
        );

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
        let url = format!(
            "{}/league/{}/standings",
            YAHOO_FANTASY_API_BASE_URL, league_key
        );

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
        let url = format!(
            "{}/league/{}/transactions",
            YAHOO_FANTASY_API_BASE_URL, league_key
        );

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
        let url = format!(
            "{}/game/{}",
            YAHOO_FANTASY_API_BASE_URL,
            game_key.to_string()
        );
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

        let url = format!("{}/games{}", YAHOO_FANTASY_API_BASE_URL, filter);

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

        let url = format!("{}/users{}", YAHOO_FANTASY_API_BASE_URL, query);

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
        let url = format!(
            "{}/league/{}",
            YAHOO_FANTASY_API_BASE_URL,
            league_key.to_string()
        );

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
            YAHOO_FANTASY_API_BASE_URL,
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
        let url = format!("{}/league/{}/teams", YAHOO_FANTASY_API_BASE_URL, league_key,);

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
        let url = format!(
            "{}/league/{}/players",
            YAHOO_FANTASY_API_BASE_URL, league_key,
        );

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
            YAHOO_FANTASY_API_BASE_URL, league_keys_str,
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
        let url = format!("{}/team/{}/roster", YAHOO_FANTASY_API_BASE_URL, team_key,);

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
