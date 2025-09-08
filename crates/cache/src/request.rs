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
}
impl<'a, T: DeserializeOwned> Request<'a, T> {
    pub fn force_update(mut self) -> Self {
        self.force_update = true;
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
            .header("Accept", "application/json");

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
