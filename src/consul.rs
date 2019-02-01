use std::{any::TypeId, fmt};

use serde_json::{from_value, json};
use reqwest::Url;

pub struct Client {
    address: Url,
    http: reqwest::Client,
}

#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error")
    }
}

impl std::error::Error for Error {}

impl From<reqwest::UrlError> for Error {
    fn from(e: reqwest::UrlError) -> Error {
        Error
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error
    }
}

impl Client {
    pub fn new(mut address: Url) -> Result<Client, Error> {
        if address.cannot_be_a_base() {
            return Err(Error);
        };
        address
            .path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv")
            .push("");
        Ok(Client {
            address,
            http: reqwest::Client::new(),
        })
    }
}

impl crate::env::Client for Client {
    type Error = Error;

    fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let mut url = self.address.join(key.trim_left_matches(|c| c == '/'))?;
        url.set_query(Some("raw=true"));
        let mut response = self.http.get(url).send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Error);
        }
        if TypeId::of::<String>() == TypeId::of::<T>() {
            Ok(from_value(json!(response.text()?))?)
        } else {
            Ok(response.json()?)
        }
    }
}
