use crate::twitter::{TimelineResponse, TweetResponse};
use regex::Regex;
use std::fmt::Display;

const DEFAULT_BASE_URL: &str = "https://syndication.twitter.com";

pub struct Client {
    base_url: String,
    cli: reqwest::Client,
}
impl Client {
    pub fn new() -> Self {
        Client {
            base_url: DEFAULT_BASE_URL.to_string(),
            cli: reqwest::Client::builder().build().unwrap(),
        }
    }

    pub async fn fetch_user_timeline<T: Display>(
        &self,
        username: T,
        cookie: T,
    ) -> Result<TimelineResponse, Box<dyn std::error::Error>> {
        let token = get_token(username.to_string().len() as u64);
        log::debug!(
            "Fetching user timeline for: {} with token: {}",
            username,
            token
        );

        let client = reqwest::Client::new();
        let req = client
            .get(format!(
                "{base_url}/srv/timeline-profile/screen-name/{username}",
                base_url = self.base_url
            ))
            .header("User-Agent", "curl/8.7.1")
            .header("Accept", "*/*")
            .header("Cookie", cookie.to_string()) // .query(&[("screen_name", username)/* , ("token", &token)*/])
            .build()?;
        log::debug!("Request URL: {}", req.url());
        let res = client.execute(req).await?;
        if res.status().is_success() {
            let html = res.text().await?;
            let re = Regex::new(
                r#"<script id="__NEXT_DATA__" type="application/json">([^>]*)</script>"#,
            )
            .unwrap();

            if let Some(caps) = re.captures(html.as_str()) {
                let json_text = &caps[1];

                Ok(serde_json::from_str::<TimelineResponse>(json_text)?)
            } else {
                return Err("Failed to find JSON data in the response".into());
            }
        } else {
            Err(format!("Failed to fetch user timeline: {}", res.status()).into())
        }
    }

    #[allow(dead_code)]
    pub async fn fetch_tweet<T: Display>(
        &self,
        tweet_id: u64,
        cookie: T,
    ) -> Result<TweetResponse, Box<dyn std::error::Error>> {
        let token = get_token(tweet_id);
        log::info!("Fetching tweet with ID: {} and token: {}", tweet_id, token);

        let client = reqwest::Client::new();
        let req = client
            .get(format!("{base_url}/tweet-result", base_url = self.base_url))
            .header("User-Agent", "curl/8.7.1")
            .header("Accept", "application/json")
            // .header("Cookie", cookie.to_string())
            .query(&[("id", &tweet_id.to_string()), ("token", &token.to_string())])
            .build()?;
        log::info!("Request URL: {}", req.url());
        let res = client.execute(req).await?;
        if res.status().is_success() {
            log::info!("Request was successful");
            Ok(res.json::<TweetResponse>().await?)
        } else {
            Err(format!("Failed to fetch tweet: {}", res.status()).into())
        }
    }
}

fn f64_to_base36_string(n: f64, precision: usize) -> String {
    let int_part = n.trunc() as u64;
    let frac_part = n.fract();
    let int_str = format!("{:}", radix_fmt::radix(int_part, 36));
    let frac_precision = precision - int_str.len() + 1;
    let mut frac = frac_part;
    let mut frac_str = String::new();
    for _ in 0..frac_precision {
        frac *= 36.0;
        let digit = frac.trunc() as u8;
        frac_str.push(std::char::from_digit(digit as u32, 36).unwrap());

        frac -= digit as f64;
    }
    format!("{}.{}", int_str, frac_str)
}

fn get_token(id: u64) -> String {
    let value = (id as f64 / 1e15) * std::f64::consts::PI;
    log::debug!("Value: {}", value);
    // let base = 36u32; // 6 ** 2 = 36
    // (value.trunc() as u64).to_string_radix(36); // Convert f64 to base36-like string
    let mut s = f64_to_base36_string(value, 11); // Convert f64 to hex as a base36-like string
    log::debug!("Hex string: {}", s);
    // Remove all '0's and '.'s
    s.retain(|c| c != '0' && c != '.');
    s
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{self, BufReader},
    };

    use super::*;

    #[test]
    fn test_decode_tweet_result() {
        let tweet_response_json = io::read_to_string(BufReader::new(
            File::open("testdata/twitter/tweet.json").expect("failed to open file"),
        ))
        .expect("failed to load file");

        let tweet_response: TweetResponse =
            serde_json::from_str(tweet_response_json.as_str()).expect("failed to deserilaize tweet");

    }

    #[test]
    fn test_get_token_black_box() {
        let tweet_id = "145634895654321";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        // assert_eq!(token, "ggyb6ihoy75");
        assert_eq!(token, ".ggyb6ihoy75");

        let tweet_id = "1609656024901861377";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        // assert_eq!(token, "3wgvt2kcol");
        assert_eq!(token, "3wg.vt2kcol");

        let tweet_id = "1955088451830710676";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        // assert_eq!(token, "4qm3alu2l4h");
        assert_eq!(token, "4qm.3alu2l4h");
    }
}
