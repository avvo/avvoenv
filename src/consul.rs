use std::any::TypeId;

use serde_json::{from_value, json};
use url::Url;

pub struct Client {
    address: Url,
    http: reqwest::Client,
}

pub struct Error;

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Error {
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
    pub fn new(address: Url) -> Result<Client, Error> {
        unimplemented!()
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
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
