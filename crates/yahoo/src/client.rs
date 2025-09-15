use crate::{
    PlayerKey,
    data::{
        DraftResult, FantasyContent, FantasyContentWrapper, Game, GameCode, GameKey, League,
        LeagueKey, Player, Roster, Scoreboard, Settings, Standings, Team, TeamKey, Token,
        Transaction, User,
    },
    token,
};
use cached_client_rs::{Request, RequestContext, ResponseBodyFormat};
use chrono::{NaiveDate, Utc};
use core::str;
use itertools::join;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use std::{
    env::{self, VarError},
    error::Error as _,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
};
use tokio::sync::Mutex;

const HEADER_XML: &str = "application/xml";
const YAHOO_CLIENT_ID: &str = "YAHOO_CLIENT_ID";
const YAHOO_REDIRECT_URI: &str = "YAHOO_REDIRECT_URI";
const YAHOO_TOKEN_FILE: &str = "YAHOO_TOKEN_FILE";
const USER_AGENT: &str = "reqwest";
const YAHOO_API_AUTH_BASE_URL: &str = "https://api.login.yahoo.com/oauth2";
const YAHOO_FANTASY_API_BASE_URL: &str = "https://fantasysports.yahooapis.com/fantasy/v2";

pub struct Client {
    base_url: String,
    auth_token_client: reqwest::Client,
    cached_client: cached_client_rs::Client,
    /// this client is responsible for generating the oauth request token url
    client_no_redirects: reqwest::Client,
    pub client_id: String,
    pub redirect_uri: String,
    code_verifier: Arc<Mutex<Option<String>>>,
    pub token_file: Option<String>,
    token: Arc<Mutex<Option<Token>>>,
    outgoing_request_count: AtomicU64,
}
impl Client {
    pub fn new_from_env() -> Result<Self, VarError> {
        let client_id = env::var(YAHOO_CLIENT_ID)?;
        let redirect_uri = env::var(YAHOO_REDIRECT_URI)?;
        Ok(Client::new(client_id, redirect_uri))
    }

    pub fn new<C: Into<String>, R: Into<String>>(client_id: C, redirect_uri: R) -> Self {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static(HEADER_XML));

        Client {
            base_url: YAHOO_FANTASY_API_BASE_URL.to_string(),
            auth_token_client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("dev error creating auth url http client for yahoo client"),
            cached_client: cached_client_rs::Client::new(YAHOO_FANTASY_API_BASE_URL.to_string())
                .with_http_client(
                    reqwest::Client::builder()
                        .user_agent(USER_AGENT)
                        .default_headers(default_headers)
                        .build()
                        .expect("dev error creating http client for yahoo client"),
                ),
            client_no_redirects: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("dev error creating http client (no redirects) for yahoo client"),
            client_id: client_id.into(),
            redirect_uri: redirect_uri.into(),
            code_verifier: Arc::new(Mutex::new(None)),
            token_file: None,
            token: Arc::new(Mutex::new(None)),
            outgoing_request_count: AtomicU64::new(0),
        }
    }

    pub fn with_base_url<U: Into<String>>(mut self, url: U) -> Self {
        self.base_url = url.into();
        self.cached_client = self.cached_client.with_base_url(self.base_url.clone());
        self
    }

    pub async fn with_cache_dir<D: Into<PathBuf>>(
        mut self,
        cache_dir: D,
    ) -> Result<Self, cached_client_rs::Error> {
        self.cached_client = self.cached_client.with_cache(cache_dir).await?;
        Ok(self)
    }

    pub fn with_token(mut self, token: Token) -> Self {
        self.token = Arc::new(Mutex::new(Some(token)));
        self
    }

    pub async fn with_token_file<F: Into<String>>(
        mut self,
        token_file: Option<F>,
    ) -> Result<Self, token::Error> {
        let token_file = token_file
            .map_or(env::var(YAHOO_TOKEN_FILE).ok(), |f| Some(f.into()))
            .ok_or(token::Error::FilenameRequired)?;
        let token = token::load_token_file(&token_file).await?;
        self.token_file = Some(token_file);
        Ok(self.with_token(token))
    }

    pub fn request_count(&self) -> u64 {
        self.outgoing_request_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn get_request_auth_url(&self) -> Result<String, Error> {
        let code_verifier = pkce::code_verifier(43);
        let code_challenge = pkce::code_challenge(&code_verifier);
        let code_challenge_method = "S256";

        let req = match self
            .client_no_redirects
            .get(format!("{}/request_auth", YAHOO_API_AUTH_BASE_URL).as_str())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("response_type", "code"),
                ("language", "en-us"),
                ("code_challenge", &code_challenge),
                ("code_challenge_method", code_challenge_method),
            ])
            .build()
        {
            Ok(request) => request,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to build GET /request_auth request: {}",
                    err
                )));
            }
        };
        let resp = match self.client_no_redirects.execute(req).await {
            Ok(response) => response,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to execute GET /request_auth request: {}",
                    err
                )));
            }
        };
        if !resp.status().is_redirection() {
            log::warn!("Failed: {}", resp.status());
            return Err(Error::UnknownError(format!(
                "Got unexpected response for GET /request_auth. Code {}",
                resp.status(),
            )));
        }

        let location = resp.headers().get("location").unwrap();

        let mut v = self.code_verifier.lock().await;
        let _ = v.insert(String::from_utf8(code_verifier).unwrap());

        Ok(location.to_str().unwrap().to_string())
    }

    pub async fn get_token(&self, code: &str) -> Result<Token, Error> {
        let code_verifier_lock = self.code_verifier.lock().await;
        let code_verifier =
            code_verifier_lock
                .as_ref()
                .ok_or(Error::UnknownError(String::from(
                    "no code_verifier available",
                )))?;

        let form = [
            ("client_id", self.client_id.as_str()),
            ("code", code),
            ("code_verifier", code_verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", "oob"),
        ];
        let req = match self
            .auth_token_client
            .post(format!("{}/get_token", YAHOO_API_AUTH_BASE_URL).as_str())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&form)
            .build()
        {
            Ok(request) => request,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to build token request: {}",
                    err
                )));
            }
        };
        self.outgoing_request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let resp = match self.auth_token_client.execute(req).await {
            Ok(response) => response,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to execute token request: {}",
                    err
                )));
            }
        };
        if !resp.status().is_success() {
            return Err(Error::UnknownError(format!(
                "Failed to get token: {}",
                resp.status()
            )));
        }
        let token_ts = Utc::now();

        // TODO: unset the codeverifier

        let mut token = resp.json::<Token>().await.map_err(|err| {
            Error::UnknownError(format!("Failed to parse response body as Token: {}", err))
        })?;
        token.ts = Some(token_ts);

        let mut token_arc = self.token.lock().await;
        let _ = token_arc.insert(token.clone());

        // save to file if filename is active
        if let Some(token_file) = &self.token_file {
            token::save_token_file(token_file, &token)
                .await
                .map_err(|err| {
                    Error::UnknownError(format!("failed to save token file after get_token: {err}"))
                })?;
        }
        Ok(token)
    }

    pub async fn refresh_token(&self, force_update: bool) -> Result<Token, Error> {
        let mut token_lock = self.token.lock().await;
        let token = token_lock
            .as_ref()
            .ok_or(Error::UnknownError("no token available".to_string()))?;

        if !force_update && !token.is_expired() {
            log::info!("not refreshing token since it's not yet within expiry window");
            return Ok(token.clone());
        }

        let req = match self
            .auth_token_client
            .post(format!("{}/get_token", YAHOO_API_AUTH_BASE_URL).as_str())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("refresh_token", token.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
                ("redirect_uri", "oob"),
            ])
            .build()
        {
            Ok(request) => request,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to build refresh token request: {}",
                    err
                )));
            }
        };
        self.outgoing_request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let resp = match self.auth_token_client.execute(req).await {
            Ok(response) => response,
            Err(err) => {
                return Err(Error::UnknownError(format!(
                    "Failed to execute refresh token request: {:?}",
                    err.source().unwrap()
                )));
            }
        };
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap();
            log::warn!("Err: body: {}", body_text);
            return Err(Error::UnknownError(format!(
                "Failed to refresh token: {}",
                status
            )));
        }
        let token_ts = Utc::now();

        let mut token = resp.json::<Token>().await.map_err(|err| {
            Error::UnknownError(format!("Failed to parse response body as Token: {}", err))
        })?;
        token.ts = Some(token_ts);

        // update arc mutex
        let _ = token_lock.insert(token.clone());

        // save to file if filename is active
        if let Some(token_file) = &self.token_file {
            token::save_token_file(token_file, &token)
                .await
                .map_err(|err| {
                    Error::UnknownError(format!("failed to save token file after refresh: {err}"))
                })?;
        }

        Ok(token)
    }

    /// Gets a copy of the token on this client
    pub(crate) async fn token(&self) -> Result<Option<Token>, Error> {
        self.token
            .lock()
            .await
            .as_ref()
            .map_or(Ok(None), |t| Ok(Some(t.clone())))
    }

    pub(crate) async fn get(
        &self,
        request_ctx: RequestContext<'_>,
    ) -> Result<FantasyContentWrapper, Error> {
        let token = self
            .token()
            .await? // should never return an err
            .ok_or(Error::UnknownError(String::from("no token available")))?;
        let req: Request<'_, FantasyContentWrapper> = self.cached_client.build_request(
            request_ctx,
            false,
            Some(ResponseBodyFormat::XML),
            Some(format!("Bearer {token}", token = token.access_token)),
        );
        req.send().await.map_err(|err| {
            match err {
                cached_client_rs::Error::CacheError(error) => {
                    Error::UnknownError(format!("failed got reqwest error: {error:?}"))
                }

                cached_client_rs::Error::ClientError(status_code, body) => {
                    let yahoo_error = serde_xml_rs::from_str::<crate::data::Error>(body.as_str())
                        .inspect_err(|err| {
                            log::warn!(
                                "got http error {status_code} and failed to parse body: {err}"
                            )
                        })
                        .ok();
                    if status_code == 401 {
                        if let Some(yahoo_error) = &yahoo_error
                            && yahoo_error
                                .description()
                                .contains("oauth_problem=\"token_expired\"")
                        {
                            // This is a special case where the token has expired.
                            // We should refresh the token and try again.
                            // log::info!("Token expired, refreshing...");
                            // match self.refresh_token().await {
                            //     Ok(new_token) => {
                            //         log::info!("New token: {:?}", new_token);
                            //         // Update the token in the client
                            //         self.token = Some(new_token);
                            //         // Retry the request with the new token
                            //         return self.get(url).await; // ooooh rescursions
                            //     }
                            //     Err(e) => {
                            //         return Err(Error::UnknownError(format!(
                            //             "Failed to refresh token: {}",
                            //             e
                            //         )));
                            //     }
                            // }
                            return Error::TokenExpired;
                        }
                        if let Some(yahoo_error) = &yahoo_error
                            && yahoo_error
                                .description()
                                .contains("oauth_problem=\"unable_to_determine_oauth_type\"")
                        {
                            return Error::OAuthError("unable_to_determine_oauth_type".to_string());
                        }
                    }
                    if let Some(yahoo_error) = &yahoo_error {
                        Error::UnknownError(format!(
                            "Got client error ({status_code}): {}",
                            yahoo_error.description()
                        ))
                    } else {
                        Error::UnknownError(format!("Got client error ({status_code}): {body}"))
                    }
                }
                cached_client_rs::Error::DevError(dev_err) => {
                    Error::UnknownError(format!("got dev error from cached_client: {dev_err}"))
                }

                cached_client_rs::Error::ReqwestError(error) => {
                    Error::UnknownError(format!("failed got reqwest error: {error:?}"))
                }

                cached_client_rs::Error::NotFound(_) => Error::NotFound,
                cached_client_rs::Error::SerdeError(serde_error) => Error::UnknownError(format!(
                    "failed to deserialize response body: {serde_error:?}"
                )),
            }
        })
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
    ) -> Result<Scoreboard, Error> {
        let request_context = RequestContext {
            url: format!(
                "/league/{league_key}/scoreboard{}",
                if let Some(week) = week {
                    format!(";week={}", week)
                } else if let Some(date) = date {
                    format!(";date={}", date)
                } else {
                    String::from("")
                }
            ),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.scoreboard {
                    Some(scoreboard) => Ok(scoreboard),
                    None => Err(Error::NotFound),
                },
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(Error::UnknownError(format!(
                "failed to fetch league scoreboard: {}",
                err
            ))),
        }
    }

    pub async fn get_league_settings(&self, league_key: &LeagueKey) -> Result<Settings, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}/settings"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.settings {
                    Some(settings) => Ok(settings),
                    None => Err(Error::NotFound),
                },
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(Error::UnknownError(format!(
                "failed to fetch league settings: {}",
                error
            ))),
        }
    }

    pub async fn get_league_standings(&self, league_key: &LeagueKey) -> Result<Standings, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}/standings"),
            filters: None,
        };
        return match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => match league.standings {
                    Some(standings) => Ok(standings),
                    None => Err(Error::NotFound),
                },
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(err),
        };
    }

    pub async fn get_league_transactions(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Vec<Transaction>, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}/transactions"),
            filters: None,
        };
        return match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(league.transactions),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(Error::UnknownError(format!(
                "failed to fetch league transactions content: {}",
                err
            ))),
        };
    }

    pub async fn get_game(&self, game_key: GameKey) -> Result<Game, Error> {
        let request_context = RequestContext {
            url: format!("/game/{game_key}"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Game(game) => Ok(game),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Game, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(Error::UnknownError(format!(
                "failed to fetch game content: {}",
                err
            ))),
        }
    }

    pub async fn get_games(
        &self,
        game_codes: Vec<GameCode>,
        seasons: Vec<&str>,
    ) -> Result<Vec<Game>, Error> {
        let mut filter = String::new();

        if !game_codes.is_empty() {
            filter.push_str(";game_codes=");
            filter.push_str(join(game_codes, ",").as_str());
        }
        if !seasons.is_empty() {
            filter.push_str(";seasons=");
            filter.push_str(seasons.join(",").as_str());
        }

        let request_context = RequestContext {
            url: format!("/games{filter}"),
            filters: None,
        };

        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Games(games) => Ok(games),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Games, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn get_users(&self, use_login: bool) -> Result<Vec<User>, Error> {
        let mut query = "";
        if use_login {
            query = ";use_login=1";
        }

        let request_context = RequestContext {
            url: format!("/users{query}"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Users(users) => Ok(users),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Users, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(error),
        }
    }

    /// league_key is {game_code}.l.{league_id}
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub async fn get_league(&self, league_key: &LeagueKey) -> Result<Option<League>, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(Some(league)),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn get_teams_for_user(&self, game_key: GameKey) -> Result<Vec<Team>, Error> {
        let request_context = RequestContext {
            url: format!("/users;use_login=1/games;game_keys={game_key}/teams",),
            filters: None,
        };

        let content = self.get(request_context).await?;

        let mut return_me: Vec<Team> = vec![];

        if let FantasyContent::Users(users) = content.content {
            for user in users {
                for game in user.games {
                    for team in game.teams {
                        return_me.push(team);
                    }
                }
            }
        } else {
            return Err(Error::UnexpectedContentVariant(format!(
                "expected FantasyContent::Users, got: {:?}",
                content.content
            )));
        }

        Ok(return_me)
    }

    pub async fn get_teams_in_league(&self, league_key: &LeagueKey) -> Result<Vec<Team>, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}/teams"),
            filters: None,
        };

        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(league.teams),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(err),
        }
    }

    ///
    /// This is tricky, I am only seeing a max returned amount of 25.
    pub async fn get_players_for_league(
        &self,
        league_key: &LeagueKey,
        player_keys_to_filter: Option<&Vec<PlayerKey>>,
    ) -> Result<Vec<Player>, Error> {
        let mut url = format!("/league/{league_key}/players");
        if let Some(player_keys) = player_keys_to_filter
            && !player_keys.is_empty()
        {
            url.push_str(";player_keys=");
            url.push_str(
                player_keys
                    .iter()
                    .map(PlayerKey::to_string)
                    .collect::<Vec<String>>()
                    .join(",")
                    .as_str(),
            );
        }
        let request_context = RequestContext { url, filters: None };

        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(league.players),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn get_players_for_leagues(
        &self,
        league_keys: Vec<&LeagueKey>,
    ) -> Result<Vec<League>, Error> {
        let league_keys_str = join(league_keys, ",");

        let url = format!("/leagues;league_keys={league_keys_str}/players");
        let request_context = RequestContext { url, filters: None };

        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Leagues(leagues) => Ok(leagues),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Leagues, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(err),
        }
    }

    /// Documentation: https://developer.yahoo.com/fantasysports/guide/#roster-resource
    ///
    /// For NFL, you can specify a week parameter to retrieve a specific week – otherwise it will default to the current roster
    /// https://fantasysports.yahooapis.com/fantasy/v2/team//roster;week=10
    ///
    /// For MLB, NHL, or NBA, you can specify a date parameter to retrieve a specific date – otherwise it will default to today’s roster.
    /// https://fantasysports.yahooapis.com/fantasy/v2/team//roster;date=2011-05-01
    pub async fn get_roster_for_team(&self, team_key: &TeamKey) -> Result<Roster, Error> {
        let request_context = RequestContext {
            url: format!("/team/{team_key}/roster"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Team(team) => match team.roster {
                    Some(roster) => Ok(roster),
                    None => Err(Error::NotFound),
                },
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Team, got: {:?}",
                    other
                ))),
            },
            Err(err) => Err(Error::UnknownError(format!("{:?}", err))),
        }
    }

    pub async fn get_rosters_in_league(&self, league_key: &LeagueKey) -> Result<Vec<Team>, Error> {
        let request_context = RequestContext {
            url: format!("/league/{league_key}/teams/roster"),
            filters: None,
        };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(league.teams),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Team, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(Error::UnknownError(format!(
                "failed to fetch team rosters for league {}: {}",
                league_key, error
            ))),
        }
    }

    pub async fn get_stats_for_team(
        &self,
        team_key: &TeamKey,
        week: Option<i32>,
        date: Option<NaiveDate>,
    ) -> Result<Roster, Error> {
        let url = format!(
            "/team/{team_key}/stats{}",
            if let Some(week) = week {
                format!(":type=week;week={}", week)
            } else if let Some(date) = date {
                format!(":type=date;date={}", date)
            } else {
                String::from("")
            }
        );
        let request_context = RequestContext { url, filters: None };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::Team(team) => match team.roster {
                    Some(roster) => Ok(roster),
                    None => Err(Error::NotFound),
                },
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::Team, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(Error::UnknownError(format!(
                "failed to fetch roster for team {}: {}",
                team_key, error
            ))),
        }
    }

    pub async fn get_draft_results(
        &self,
        league_key: &LeagueKey,
    ) -> Result<Vec<DraftResult>, Error> {
        let url = format!("/league/{league_key}/draftresults");
        let request_context = RequestContext { url, filters: None };
        match self.get(request_context).await {
            Ok(content) => match content.content {
                FantasyContent::League(league) => Ok(league.draft_results),
                other => Err(Error::UnexpectedContentVariant(format!(
                    "expected FantasyContent::League, got: {:?}",
                    other
                ))),
            },
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{Client, GameCode, GameKey, LeagueKey, TeamKey, Token, data::DraftStatus};
    use chrono::{NaiveDate, Utc};
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

        let fixture = Client::new("", "")
            .with_base_url(mock_server.uri())
            .with_token(Token {
                access_token: "foo".to_string(),
                token_type: "bearer".to_string(),
                expires_in: 3600,
                refresh_token: "bar".to_string(),
                scope: None,
                ts: Some(Utc::now()),
            });

        (fixture, mock_server)
    }
    async fn setup_with_error(p: &str, code: u16, error: &str) -> (Client, MockServer) {
        let mock_server = wiremock::MockServer::builder().start().await;
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(
                ResponseTemplate::new(code)
                    .set_body_string(
                        fs::read_to_string(format!("tests/data/fantasy/v2/errors/{}.xml", error))
                            .unwrap(),
                    )
                    .insert_header("Content-Type", "application/xml"),
                // < HTTP/2 401
                // < date: Sun, 25 May 2025 14:51:07 GMT
                // < server: ATS
                // < www-authenticate: OAuth oauth_problem="token_expired", realm="yahooapis.com"
                // < content-length: 268
                // < content-type: application/xml
                // < x-envoy-upstream-service-time: 1
                // < cache-control: max-age=0, private
                // < expires: -1
                // < age: 0
                // < strict-transport-security: max-age=31536000
                // < referrer-policy: strict-origin-when-cross-origin
                // < x-content-type-options: nosniff
                // < x-xss-protection: 1; mode=block
                // < x-frame-options: SAMEORIGIN
            )
            .mount(&mock_server)
            .await;

        let fixture = Client::new("", "")
            .with_base_url(mock_server.uri())
            .with_token(Token {
                access_token: "foo".to_string(),
                token_type: "bearer".to_string(),
                expires_in: 3600,
                refresh_token: "bar".to_string(),
                scope: None,
                ts: Some(Utc::now()),
            });
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
        assert_eq!(0, games[0].teams.len());
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
        let cases = vec![
            ("2009", 222),
            ("2010", 242),
            ("2011", 257),
            ("2012", 273),
            ("2013", 314),
            ("2014", 331),
            ("2015", 348),
            ("2016", 359),
            ("2017", 371),
            ("2018", 380),
            ("2019", 390),
            ("2020", 399),
            ("2021", 406),
            ("2022", 414),
            ("2023", 423),
            ("2024", 449),
            ("2025", 461),
        ];

        let (fixture, _mock_server) = setup_with_200("/games;game_codes=nfl;seasons=2009,2010,2011,2012,2013,2014,2015,2016,2017,2018,2019,2020,2021,2022,2023,2024,2025").await;
        let games = fixture
            .get_games(vec![GameCode::NFL], cases.iter().map(|(s, _)| *s).collect())
            .await
            .expect("should return ok (although should be empty)");

        assert_eq!(17, games.len());
        // all are NFL
        for (i, game) in games.iter().enumerate() {
            assert_eq!(GameCode::NFL, game.code);
            assert_eq!("Football", game.name);
            debug_assert_eq!(cases[i].0, game.season, "season does not match");
            debug_assert_eq!(
                cases[i].1,
                game.game_id,
                "game id does not match for {szn}",
                szn = game.season
            );
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

        assert_eq!(
            TeamKey::new(LeagueKey::new(GameKey::Id(390), 299227), 1),
            teams[0].team_key
        );
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
        let league_key = LeagueKey::new(GameKey::Id(380), 129397);
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
                &LeagueKey::new(GameKey::Id(390), 299227), // 2019 nfl
            )
            .await
            .expect("should return ok");

        let league = opt_league.expect("should be there");
        assert_eq!(LeagueKey::new(GameKey::Id(390), 299227), league.league_key);
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

    #[tokio::test]
    async fn test_get_league_standings_incomplete() {
        let league_key = LeagueKey::new(GameKey::Id(390), 299227);
        let (fixture, _mock_server) = setup_with_200("/league/390.l.299227/standings").await;
        let standings = fixture
            .get_league_standings(
                &league_key, // 2019 nfl
            )
            .await
            .expect("should return ok");
        assert_eq!(1, standings.teams.len())
    }

    #[tokio::test]
    async fn test_get_league_standings_token_expired() {
        let league_key = LeagueKey::new(GameKey::Id(390), 299227);
        let (fixture, _mock_server) =
            setup_with_error("/league/390.l.299227/standings", 401, "token_expired").await;
        let resp = fixture
            .get_league_standings(
                &league_key, // 2019 nfl
            )
            .await;
        assert!(resp.is_err());
        let error = resp.err().unwrap();
        assert_eq!(crate::client::Error::TokenExpired, error);
    }

    #[tokio::test]
    async fn test_get_roster_players_for_team() {
        let league_key = LeagueKey::new(GameKey::Id(390), 299227);
        let (fixture, _mock_server) = setup_with_200("/team/390.l.299227.t.1/roster").await;
        let roster = fixture
            .get_roster_for_team(&TeamKey::new(league_key, 1))
            .await
            .expect("request failed");
        assert_eq!(Some("16".to_string()), roster.week);
    }

    #[tokio::test]
    async fn test_get_rosters_for_league() {
        let league_key = LeagueKey::new(GameKey::Id(380), 129397);
        let (fixture, _mock_server) = setup_with_200("/league/380.l.129397/teams/roster").await;
        let teams_w_roster = fixture
            .get_rosters_in_league(&league_key)
            .await
            .expect("request failed");
        assert_eq!(12, teams_w_roster.len());
    }

    #[tokio::test]
    async fn test_get_draft_results_for_league() {
        let league_key = LeagueKey::new(GameKey::Id(380), 129397);
        let (fixture, _mock_server) = setup_with_200("/league/380.l.129397/draftresults").await;
        let draft_results = fixture
            .get_draft_results(&league_key)
            .await
            .expect("request failed");
        assert_eq!(204, draft_results.len());
        assert_eq!(1, draft_results[0].pick);
    }

    #[tokio::test]
    async fn test_get_transactions_for_league() {
        let league_key = LeagueKey::new(GameKey::Id(359), 564503);
        let (fixture, _mock_server) = setup_with_200("/league/359.l.564503/transactions").await;
        let transactions = fixture
            .get_league_transactions(&league_key)
            .await
            .expect("request failed");
        assert_eq!(323, transactions.len());
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    // #[error("Request error: {0}")]
    // RequestError(#[from] reqwest::Error),
    // #[error("Deserialization error: {0}")]
    // DeserializationError(#[from] serde_xml_rs::Error),
    #[error("Unexpected content variant: {0}")]
    UnexpectedContentVariant(String),
    #[error("token expired, please refresh")]
    TokenExpired,
    #[error("OAuth error: {0}")]
    OAuthError(String),
    #[error("resource not found")]
    NotFound,
    #[error("unknown error: {0}")]
    UnknownError(String),
}
