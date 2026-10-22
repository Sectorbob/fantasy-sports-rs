mod data;

use scraper::{Html, Selector};

pub use data::ECRResponse;

const DEFAULT_BASE_URL: &str = "https://www.fantasypros.com";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:138.0) Gecko/20100101 Firefox/138.0";

pub struct Client {
    cli: reqwest::Client,
    base_url: String,
}
impl Client {
    pub fn new() -> Self {
        Client::new_with_base_url(DEFAULT_BASE_URL)
    }
    pub fn new_with_base_url<T: Into<String>>(base_url: T) -> Self {
        Client {
            cli: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .gzip(true)
                .build()
                .unwrap(),
            base_url: base_url.into(),
        }
    }
    pub async fn fetch_ecr_half_ppr(&self) -> Result<ECRResponse, FantasyProsError> {
        let resp = self
            .cli
            .get(format!(
                "{base_url}/nfl/rankings/half-point-ppr-cheatsheets.php",
                base_url = self.base_url
            ))
            .send()
            .await?;
        let html = resp.text().await?;
        parse_html(html)
    }
}

const ECR_LINE_PREFIX: &str = "var ecrData = ";

pub(crate) fn parse_html(html: String) -> Result<ECRResponse, FantasyProsError> {
    let html = Html::parse_document(html.as_str());
    let selector = Selector::parse("script").unwrap();
    let mut matches = html
        .select(&selector)
        .filter_map(|element| {
            if element.attr("type") != Some("text/javascript") {
                return None;
            }
            element
                .inner_html()
                .lines()
                .map(&str::trim) // trim off leading and trailing whitespace
                .find(|line| line.starts_with(ECR_LINE_PREFIX))
                .map(|line| {
                    let tmp = line.strip_prefix(ECR_LINE_PREFIX).unwrap();
                    tmp.strip_suffix(";").unwrap_or(tmp).to_string()
                })
        })
        .collect::<Vec<String>>();
    Ok(serde_json::from_str(matches.remove(0).as_str())?)
}

#[derive(thiserror::Error, Debug)]
pub enum FantasyProsError {
    #[error("http error occurred {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("failed to decode embdedded json {0}")]
    InvalidJson(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use wiremock::{Mock, ResponseTemplate, matchers};

    use crate::fantasypros::Client;

    #[tokio::test]
    async fn test_fetch_rankings() {
        // setup
        let mock_server = wiremock::MockServer::builder().start().await;
        mock_server
            .register(setup_mock(
                "/nfl/rankings/half-point-ppr-cheatsheets.php",
                "testdata/fantasypros/synthetic_rankings_half_ppr.html.gz",
            ))
            .await;
        let client = Client::new_with_base_url(mock_server.uri());

        // perform
        let resp = client.fetch_ecr_half_ppr().await.expect("request failed");

        // assert
        debug_assert_eq!(10, resp.players.len());
        assert_eq!(1736943600, resp.last_updated_ts);
    }

    fn setup_mock(path: &str, data_file: &str) -> Mock {
        Mock::given(matchers::method("GET"))
            .and(matchers::path(path))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header("age", "329")
                    .append_header("cache-control", "max-age=600, must-revalidate")
                    .append_header("content-encoding", "gzip")
                    .append_header("content-language", "en")
                    .append_header("content-type", "text/html; charset=UTF-8")
                    .append_header("server", "Apache/2.4.52 (Ubuntu)")
                    .append_header("x-xss-protection", "1; mode=block")
                    .set_body_bytes(
                        fs::read(data_file)
                            .unwrap_or_else(|_| panic!("failed to load file: {data_file}")),
                    ),
            )
    }
}
