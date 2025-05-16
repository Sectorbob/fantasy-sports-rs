use crate::data::{FantasyContent, Game, GameCode, Games, GameKey, Scoreboard, Token, User};
use core::str;
use itertools::join;
use serde_xml_rs::from_str;

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
            ("refresh_token", self.token.as_mut().unwrap().refresh_token.as_str()),
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
            panic!("Failed to refresh token: {}", resp.status());
        }

        let body = resp.json::<Token>().await?;
        self.token = Some(body.clone());
        return Ok(body);
    }

    pub async fn get(&self, url: &str) -> Result<FantasyContent, reqwest::Error> {
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
            let body = resp.text().await.expect("couldn't read response body");
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

        let fantasy_content = from_str(body_text.as_str()).unwrap();

        Ok(fantasy_content)
    }

    pub async fn get_league_scoreboard(
        &self,
        league_key: &str,
    ) -> Result<Scoreboard, reqwest::Error> {
        let url = format!(
            "{}/league/{}/scoreboard",
            YAHOO_FANTASY_API_BASE_URL, league_key
        );
        Ok(self
            .get(url.as_str())
            .await
            .unwrap()
            .league
            .unwrap()
            .scoreboard)
    }

    pub async fn get_game(&self, game_key: GameKey) -> Result<Game, reqwest::Error> {
        let url = format!(
            "{}/game/{}",
            YAHOO_FANTASY_API_BASE_URL,
            game_key.to_string()
        );
        Ok(self.get(url.as_str()).await.unwrap().game.unwrap())
    }

    pub async fn get_games(
        &self,
        game_codes: Vec<GameCode>,
        seasons: Vec<&str>,
    ) -> Result<Games, reqwest::Error> {
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
        Ok(self.get(url.as_str()).await.unwrap().games.unwrap())
    }

    pub async fn get_users(&self, use_login: bool) -> Result<Vec<User>, reqwest::Error> {
        let mut query = "";
        if use_login {
            query = ";use_login=1";
        }

        let url = format!("{}/users{}", YAHOO_FANTASY_API_BASE_URL, query);
        Ok(self.get(url.as_str()).await.unwrap().users.unwrap().users)
    }
}
