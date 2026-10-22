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

    #[allow(dead_code)]
    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into();
        self
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

        let req = self
            .cli
            .get(format!(
                "{base_url}/srv/timeline-profile/screen-name/{username}",
                base_url = self.base_url
            ))
            .header("User-Agent", "curl/8.7.1")
            .header("Accept", "*/*")
            .header("Cookie", cookie.to_string()) // .query(&[("screen_name", username)/* , ("token", &token)*/])
            .build()?;
        log::debug!("Request URL: {}", req.url());
        let res = self.cli.execute(req).await?;
        if res.status().is_success() {
            let html = res.text().await?;
            // tokio::fs::File::create("timeline.html")
            //     .await?
            //     .write_all(html.as_bytes())
            //     .await?;
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
    pub async fn fetch_tweet(
        &self,
        tweet_id: u64,
    ) -> Result<TweetResponse, Box<dyn std::error::Error>> {
        let token = get_token(tweet_id);
        log::info!("Fetching tweet with ID: {} and token: {}", tweet_id, token);

        let req = self
            .cli
            .get(format!("{base_url}/tweet-result", base_url = self.base_url))
            .header("User-Agent", "curl/8.7.1")
            .header("Accept", "application/json")
            .query(&[("id", &tweet_id.to_string()), ("token", &token.to_string())])
            .build()?;
        log::info!("Request URL: {}", req.url());
        let res = self.cli.execute(req).await?;
        if res.status().is_success() {
            log::info!("Request was successful");
            let text = res.text().await?;
            // tokio::fs::File::create("tweet_response.json")
            //     .await?
            //     .write_all(text.as_bytes())
            //     .await?;
            Ok(serde_json::from_str(text.as_str())?)
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
    use chrono::DateTime;
    use tokio::fs::{self};
    use wiremock::{
        Mock, ResponseTemplate,
        matchers::{self},
    };

    use super::*;

    #[tokio::test]
    async fn test_fetch_timeline() {
        let twitter_user = "SampleScribe";
        let test_data_file = "testdata/timeline.html";

        // setup
        let mock_server = wiremock::MockServer::builder().start().await;
        mock_server
            .register(
                Mock::given(matchers::method("GET"))
                    .and(matchers::path(format!(
                        "/srv/timeline-profile/screen-name/{twitter_user}"
                    )))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .append_header("content-language", "en")
                            .append_header("content-type", "text/html; charset=UTF-8")
                            .set_body_bytes(
                                fs::read(test_data_file).await.expect(
                                    format!("failed to load file: {test_data_file}").as_str(),
                                ),
                            ),
                    ),
            )
            .await;
        let fixture = Client::new().with_base_url(mock_server.uri());

        // execute
        let timeline = fixture
            .fetch_user_timeline(twitter_user, "")
            .await
            .expect("failed to fetch twitter timeline");

        // assert
        debug_assert!(
            mock_server.received_requests().await.is_some(),
            "mock not called"
        );
        debug_assert_eq!(
            5,
            timeline.props.page_props.timeline.entries.len(),
            "did not receive the expected number of tweets"
        );
        // for tweet in timeline.tweets() {
        //     println!("Entry: {tweet:#?}");
        // }
        // assert!(false)
    }

    #[tokio::test]
    async fn test_decode_tweet_result() {
        let test_data_files = vec![
            // "testdata/twitter/tweet.json",
            "testdata/tweet_response.json",
        ];
        for filename in test_data_files {
            let tweet_response_json = fs::read_to_string(filename)
                .await
                .expect("failed to open file");

            let tweet_response: TweetResponse = serde_json::from_str(tweet_response_json.as_str())
                .expect("failed to deserialize tweet");
            assert_eq!(
                DateTime::parse_from_rfc3339("2024-01-01T21:01:16.000Z").unwrap(),
                tweet_response.created_at
            )
        }
    }

    #[test]
    fn test_get_token_black_box() {
        let tweet_id = "145634895654321";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        assert_eq!(token, "ggyb6ihoy75");
        // assert_eq!(token, ".ggyb6ihoy75");

        let tweet_id = "1609656024901861377";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        assert_eq!(token, "3wgvt2kcol");
        // assert_eq!(token, "3wg.vt2kcol");

        let tweet_id = "1955088451830710676";
        let token = get_token(tweet_id.parse::<u64>().unwrap());
        assert_eq!(token, "4qm3alu2l4h");
        // assert_eq!(token, "4qm.3alu2l4h");
    }
}
