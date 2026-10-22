use crate::{Client, Error};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub struct RequestContext<'a> {
    pub url: String,
    pub filters: Option<Vec<(&'a str, String)>>,
}

pub struct Request<'a, T: DeserializeOwned> {
    pub(crate) cli: &'a Client,
    pub(crate) force_update: bool,
    pub(crate) url: String,
    pub(crate) filters: Option<Vec<(&'a str, String)>>,
    pub(crate) phantom: PhantomData<&'a T>,
    pub(crate) response_body_format: ResponseBodyFormat,
    pub(crate) bearer_token: Option<String>,
}
impl<'a, T: DeserializeOwned> Request<'a, T> {
    pub fn force_update(mut self) -> Self {
        self.force_update = true;
        self
    }

    /// Sends `Authorization: Bearer <token>` with this request (marked sensitive, so it is
    /// redacted from debug output). The token is not part of the cache key.
    pub fn with_bearer_auth<S: Into<String>>(mut self, token: S) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    pub fn with_force_update(mut self, force_update: &bool) -> Self {
        self.force_update = *force_update;
        self
    }

    pub(crate) fn build_http_request(&self) -> reqwest::Result<reqwest::Request> {
        let mut request_builder = self
            .cli
            .http_client
            .get(&self.url)
            .header("Accept", self.response_body_format.as_mime_str());

        if let Some(token) = &self.bearer_token {
            request_builder = request_builder.bearer_auth(token);
        }
        if let Some(filters) = &self.filters {
            request_builder = request_builder.query(&filters)
        }
        request_builder.build()
    }

    pub async fn send(self) -> Result<T, Error> {
        let req = self.build_http_request()?;
        self.cli.get(req, self.force_update).await
    }
}

#[derive(Default)]
pub enum ResponseBodyFormat {
    #[default]
    JSON,
    XML,
}
impl ResponseBodyFormat {
    pub fn as_mime_str(&self) -> &'static str {
        match self {
            ResponseBodyFormat::JSON => "application/json",
            ResponseBodyFormat::XML => "application/xml",
        }
    }
}
