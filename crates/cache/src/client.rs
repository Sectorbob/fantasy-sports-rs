use crate::{FileSystemCacher, Request, RequestContext, ResponseBodyFormat, cache::Cacheable};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, de::DeserializeOwned};
use std::{marker::PhantomData, path::PathBuf, sync::atomic::AtomicU64};

/// Client provides base functionality for a REST-ful GET client that can check
/// a local cache dir. Cached responses expire after a time to live (24 hours by
/// default); a cache skip can also be forced with the force_update flag
pub struct Client {
    pub(crate) http_client: reqwest::Client,
    base_url: String,
    cache: Option<Box<dyn Cacheable + Send>>,
    outgoing_request_count: AtomicU64,
}
impl Client {
    /// Instantiates a new instance of the client
    pub fn new<T: Into<String>>(base_url: T) -> Self {
        Client {
            http_client: reqwest::Client::new(),
            base_url: base_url.into(),
            cache: None,
            outgoing_request_count: AtomicU64::new(0),
        }
    }

    /// Builder function that updates the base_url on the client to the value
    /// provided.
    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// with_cache creates/opens a file_system cache using the root file path
    /// provided with cache_dir
    pub fn with_cache<T: Into<PathBuf>>(mut self, cache_dir: T) -> Result<Self> {
        let cache = FileSystemCacher::open_sync(cache_dir).map_err(Error::cache_error)?;
        self.cache = Some(Box::new(cache));
        Ok(self)
    }

    /// with_http_client is a builder method that will update the reqwest http
    /// client with custom functionality. THis can be done to add default
    /// headers, and other reqwest client builder settings.
    pub fn with_http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = http_client;
        self
    }

    /// Default behavior to build the http request and requires the expected
    /// response body type.
    pub fn build_request<'a, T: DeserializeOwned>(
        &'a self,
        ctx: RequestContext<'a>,
        force_update: bool,
        response_data_format: Option<ResponseBodyFormat>,
    ) -> Request<'a, T> {
        let url = format!("{base_url}{path}", base_url = self.base_url, path = ctx.url);

        Request {
            cli: self,
            force_update,
            url,
            filters: ctx.filters,
            phantom: PhantomData,
            response_body_format: response_data_format.unwrap_or_default(),
            bearer_token: None,
        }
    }

    /// Executes the request.exec and does the proper cache check, if a remote
    /// is hit and receives a 2XX level error then this will cache the
    /// response.
    pub async fn get<T: DeserializeOwned>(
        &self,
        req: reqwest::Request,
        force_update: bool,
    ) -> Result<T> {
        let url = req.url().to_string();
        let req_accept_mime = Client::check_accept_mime_type(req.headers())?;

        // check cache if exists (TODO: some time-based logic)
        if force_update {
            log::info!("GET {url} force skip cache");
        } else if let Some(cacher) = &self.cache {
            if let Some(bytes) = cacher.load_cache(url.as_str()).await {
                log::debug!("GET {url} cache hit");
                return self.deserialize_slice(req_accept_mime, &bytes);
            } else {
                log::info!("GET {url} cache miss");
            }
        }

        self.outgoing_request_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let resp = self.http_client.execute(req).await?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                if self.cache.is_some() {
                    self.cache_resp_body(url, resp).await
                } else {
                    let mime =
                        Client::check_content_mime_type(resp.headers()).unwrap_or(req_accept_mime);
                    let b = resp.bytes().await?;
                    self.deserialize_slice::<Option<T>>(mime, &b)?
                        .ok_or(Error::NotFound(url))
                }
            }
            reqwest::StatusCode::NOT_FOUND => Err(Error::NotFound(url)),
            status => {
                // keep the body: APIs such as Yahoo's put the reason for a 401 in it
                let body = resp.text().await.unwrap_or_default();
                Err(Error::ClientError { status, body })
            }
        }
    }

    pub async fn cache_resp_body<T: DeserializeOwned>(
        &self,
        url: String,
        resp: reqwest::Response,
    ) -> Result<T> {
        // load to memory buffer then save to file and return deserialized struct
        let mime = Client::check_content_mime_type(resp.headers())?;
        let resp_bytes = resp.bytes().await?;
        if let Some(cacher) = &self.cache {
            let cache_result = cacher.save_cache(&url, &resp_bytes).await; // TODO: handle error case
            if let Err(err) = cache_result {
                // log it
                log::error!("unable to cache sleeper GET response for {url}: {err:?}");
            }
            self.deserialize_slice(mime, &resp_bytes)
        } else {
            Err(Error::dev_error("cacher not enabled"))
        }
    }

    pub fn deserialize_slice<'a, T: for<'de> Deserialize<'de>>(
        &self,
        fmt: ResponseBodyFormat,
        b: &'a [u8],
    ) -> Result<T> {
        match fmt {
            ResponseBodyFormat::JSON => {
                serde_json::from_slice(b).map_err(|err| Error::SerdeError(SerdeError::from(err)))
            }
            ResponseBodyFormat::XML => quick_xml::de::from_reader(b)
                .map_err(|err| Error::SerdeError(SerdeError::from(err))),
        }
    }

    pub fn check_accept_mime_type(
        headers: &reqwest::header::HeaderMap,
    ) -> Result<ResponseBodyFormat> {
        match headers.get(ACCEPT) {
            Some(mime) => match mime.to_str() {
                Ok(mime) => match mime {
                    "application/json" => Ok(ResponseBodyFormat::JSON),
                    "application/xml" => Ok(ResponseBodyFormat::XML),
                    other => Err(Error::DevError(format!(
                        "{CONTENT_TYPE}: {other} is not supported"
                    ))),
                },
                Err(header_value_err) => Err(Error::DevError(format!(
                    "{CONTENT_TYPE} header value is invalid: {header_value_err}"
                ))),
            },
            None => Err(Error::DevError(format!(
                "no {CONTENT_TYPE} provided by response"
            ))),
        }
    }
    pub fn check_content_mime_type(
        headers: &reqwest::header::HeaderMap,
    ) -> Result<ResponseBodyFormat> {
        match headers.get(CONTENT_TYPE) {
            Some(mime) => match mime.to_str() {
                Ok(mime) => match mime {
                    "application/json" => Ok(ResponseBodyFormat::JSON),
                    "application/xml" => Ok(ResponseBodyFormat::XML),
                    other => Err(Error::DevError(format!(
                        "{CONTENT_TYPE}: {other} is not supported"
                    ))),
                },
                Err(header_value_err) => Err(Error::DevError(format!(
                    "{CONTENT_TYPE} header value is invalid: {header_value_err}"
                ))),
            },
            None => Err(Error::DevError(format!(
                "no {CONTENT_TYPE} provided by response"
            ))),
        }
    }

    /// Retrieve the count of the outgoing http requests
    pub fn request_count(&self) -> u64 {
        self.outgoing_request_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Error is custom
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cache error: {0}")]
    CacheError(std::io::Error),
    #[error("client error: {status}")]
    ClientError {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("dev error: {0}")]
    DevError(String),
    #[error("http error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("serde error: {0}")]
    SerdeError(#[from] SerdeError),
}
impl Error {
    pub fn dev_error<T: Into<String>>(msg: T) -> Self {
        Error::DevError(msg.into())
    }
    pub fn cache_error(err: std::io::Error) -> Self {
        Error::CacheError(err)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SerdeError {
    #[error("json deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("xml deserialization error: {0}")]
    XMLError(#[from] quick_xml::DeError),
}

#[cfg(test)]
mod tests {
    use crate::{Client, Error, RequestContext};
    use serde_json::{Value, json};
    use tempfile::TempDir;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

    fn ctx(url: &str) -> RequestContext<'static> {
        RequestContext {
            url: url.to_string(),
            filters: None,
        }
    }

    #[tokio::test]
    async fn non_success_responses_keep_the_status_and_body() {
        let server = MockServer::start().await;
        Mock::given(matchers::path("/denied"))
            .respond_with(ResponseTemplate::new(401).set_body_string("token_expired"))
            .mount(&server)
            .await;
        let client = Client::new(server.uri());

        let err = client
            .build_request::<Value>(ctx("/denied"), false, None)
            .send()
            .await
            .unwrap_err();

        match err {
            Error::ClientError { status, body } => {
                assert_eq!(401, status.as_u16());
                assert_eq!("token_expired", body);
            }
            other => panic!("expected ClientError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn bearer_token_is_sent_only_when_requested() {
        let server = MockServer::start().await;
        Mock::given(matchers::path("/secure"))
            .and(matchers::header("authorization", "Bearer s3cret"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"ok": true}))
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        Mock::given(matchers::path("/secure"))
            .respond_with(ResponseTemplate::new(401).set_body_string("no token"))
            .mount(&server)
            .await;
        let client = Client::new(server.uri());

        let v: Value = client
            .build_request(ctx("/secure"), false, None)
            .with_bearer_auth("s3cret")
            .send()
            .await
            .unwrap();
        assert_eq!(json!({"ok": true}), v);

        let err = client
            .build_request::<Value>(ctx("/secure"), false, None)
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::ClientError { status, .. } if status.as_u16() == 401));
    }

    #[tokio::test]
    async fn not_found_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(matchers::path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let client = Client::new(server.uri());

        let err = client
            .build_request::<Value>(ctx("/missing"), false, None)
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(url) if url.ends_with("/missing")));
    }

    #[tokio::test]
    async fn second_request_is_served_from_the_cache_unless_forced() {
        let server = MockServer::start().await;
        Mock::given(matchers::path("/thing"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"n": 1}))
                    .insert_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;
        let dir = TempDir::new().unwrap();
        let client = Client::new(server.uri()).with_cache(dir.path()).unwrap();

        for _ in 0..2 {
            let v: Value = client
                .build_request(ctx("/thing"), false, None)
                .send()
                .await
                .unwrap();
            assert_eq!(json!({"n": 1}), v);
        }
        assert_eq!(
            1,
            client.request_count(),
            "second call should hit the cache"
        );

        let _: Value = client
            .build_request(ctx("/thing"), false, None)
            .force_update()
            .send()
            .await
            .unwrap();
        assert_eq!(2, client.request_count(), "force_update bypasses the cache");
    }

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
                None,
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
                    url: "/players/nfl/trending/drop".to_string(),
                    filters: Some(vec![("limit", "100".to_string())]),
                },
                false,
                None,
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
                    url: "/players/nfl/trending/drop".to_string(),
                    filters: Some(vec![
                        ("lookback_hours", "10".to_string()),
                        ("limit", "100".to_string()),
                    ]),
                },
                false,
                None,
            )
            .build_http_request()
            .expect("failed to build request");
        assert_eq!(
            format!("{base_url}/players/nfl/trending/drop?lookback_hours=10&limit=100"),
            request_with_limit_and_lookback.url().to_string(),
        );
    }
}
