use crate::{FileSystemCacher, Request, RequestContext, cache::Cacheable};
use serde::de::DeserializeOwned;
use std::{marker::PhantomData, path::PathBuf, sync::atomic::AtomicU64};

pub struct Client {
    pub(crate) http_client: reqwest::Client,
    base_url: String,
    cache: Option<Box<dyn Cacheable + Send + Sync>>,
    outgoing_request_count: AtomicU64,
}
impl Client {
    pub fn new<T: Into<String>>(base_url: T) -> Self {
        Client {
            http_client: reqwest::Client::new(),
            base_url: base_url.into(),
            cache: None,
            outgoing_request_count: AtomicU64::new(0),
        }
    }

    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_cache<T: Into<PathBuf>>(mut self, cache_dir: T) -> Self {
        self.cache = Some(Box::new(FileSystemCacher::new(cache_dir)));
        self
    }

    pub fn with_http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = http_client;
        self
    }

    pub fn build_request<'a, T: DeserializeOwned>(
        &'a self,
        ctx: RequestContext<'a>,
        force_update: bool,
    ) -> Request<'a, T> {
        let url = format!("{base_url}{path}", base_url = self.base_url, path = ctx.url);

        Request {
            cli: self,
            force_update,
            url,
            filters: ctx.filters,
            phantom: PhantomData,
        }
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        req: reqwest::Request,
        force_update: bool,
    ) -> Result<T, Error> {
        let url = req.url().to_string();

        // check cache if exists (TODO: some time-based logic)
        if force_update {
            log::info!("GET {url} force skip cache");
        } else if let Some(cacher) = &self.cache {
            if let Some(body_text) = cacher.load_cache(url.as_str()).await {
                log::debug!("GET {} cache hit", url);
                return Ok(serde_json::from_str(&body_text)?);
            } else {
                log::info!("GET {} cache miss", url);
            }
        }

        self.outgoing_request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let resp = self.http_client.execute(req).await?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                if let Some(cacher) = &self.cache {
                    // load to memory buffer then save to file and return deserialized struct
                    let resp_text = resp.text().await?;
                    let cache_result = cacher.save_cache(&url, resp_text.as_str()).await; // TODO: handle error case
                    if let Err(err) = cache_result {
                        // log it
                        log::error!("unable to cache sleeper GET response for {url}: {:?}", err);
                    }
                    return Ok(serde_json::from_str(&resp_text)?);
                } else {
                    resp.json::<Option<T>>().await?.ok_or(Error::NotFound(url))
                }
            }
            reqwest::StatusCode::NOT_FOUND => Err(Error::NotFound(url)),
            code => Err(Error::from(code)),
        }
    }

    pub fn request_count(&self) -> u64 {
        self.outgoing_request_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("client error: {0}")]
    ClientError(reqwest::StatusCode),
    #[error("dev error: {0}")]
    DevError(String),
    #[error("http error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("json deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}
impl Error {
    pub fn dev_error<T: Into<String>>(msg: T) -> Self {
        Error::DevError(msg.into())
    }
}
impl From<reqwest::StatusCode> for Error {
    fn from(value: reqwest::StatusCode) -> Self {
        Error::ClientError(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;
    use serde_json::Value;

    #[test]
    fn test_build_request() {
        let base_url = "http://localhost:1234";
        let fixture = Client::new(base_url);
        let request_with_lookback = fixture
            .build_request::<Value>(
                crate::RequestContext {
                    url: "/players/nfl/trending/drop".to_string(),
                    filters: Some(vec![("lookback_hours", "6".to_string())]),
                },
                false,
            )
            .build_http_request()
            .expect("failed to build request");
        assert_eq!(
            format!("{base_url}/players/nfl/trending/drop?lookback_hours=6",),
            request_with_lookback.url().to_string()
        );
        let request_with_limit = fixture
            .build_request::<Value>(
                crate::RequestContext {
                    url: format!("/players/nfl/trending/drop"),
                    filters: Some(vec![("limit", "100".to_string())]),
                },
                false,
            )
            .build_http_request()
            .expect("failed to build request");
        assert_eq!(
            format!("{base_url}/players/nfl/trending/drop?limit=100"),
            request_with_limit.url().to_string(),
        );
        let request_with_limit_and_lookback = fixture
            .build_request::<Value>(
                crate::RequestContext {
                    url: format!("/players/nfl/trending/drop"),
                    filters: Some(vec![
                        ("lookback_hours", "10".to_string()),
                        ("limit", "100".to_string()),
                    ]),
                },
                false,
            )
            .build_http_request()
            .expect("failed to build request");
        assert_eq!(
            format!("{base_url}/players/nfl/trending/drop?lookback_hours=10&limit=100"),
            request_with_limit_and_lookback.url().to_string(),
        );
    }
}
