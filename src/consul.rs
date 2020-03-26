use std::{any::TypeId, fmt};

use log::trace;
use reqwest::Url;
use serde_json::{from_value, json};

use crate::client_error::ClientError;

#[derive(Debug)]
pub struct Client {
    address: Url,
    http: reqwest::blocking::Client,
}

#[derive(Debug)]
pub struct Error(ClientError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl<T: Into<ClientError>> From<T> for Error {
    fn from(e: T) -> Error {
        Error(e.into())
    }
}

impl Client {
    pub fn new(mut address: Url) -> Result<Client, Error> {
        if address.cannot_be_a_base() {
            return Err(ClientError::BaseUrlError(address).into());
        };
        address
            .path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv")
            .push("");
        Ok(Client {
            address,
            http: reqwest::blocking::Client::new(),
        })
    }
}

impl crate::env::Client for Client {
    type Error = Error;

    fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let mut url = self.address.join(key.trim_start_matches(|c| c == '/'))?;
        url.set_query(Some("raw=true"));
        let request = self.http.get(url.clone());
        trace!("{:?}", request);
        let response = request
            .send()
            .map_err(|e| ClientError::with_url(url.clone(), e))?;
        trace!("{:?}", response);
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(ClientError::ServerError(response).into());
        }
        if TypeId::of::<String>() == TypeId::of::<T>() {
            let body = response
                .text()
                .map_err(|e| ClientError::with_url(url.clone(), e))?;
            from_value(json!(body)).map_err(|e| ClientError::with_url(url, e).into())
        } else {
            Ok(response.json().map_err(|e| ClientError::with_url(url, e))?)
        }
    }
}
