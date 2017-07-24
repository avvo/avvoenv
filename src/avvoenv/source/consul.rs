use std::result::Result;
use std::io::Read;

use avvoenv::errors;
use avvoenv::source::Source;

extern crate reqwest;
extern crate serde;

pub struct Client {
    address: reqwest::Url,
    http: reqwest::Client,
}

impl Client {
    pub fn new(mut address: reqwest::Url) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv");
        let client = match reqwest::Client::new() {
            Ok(value) => value,
            Err(_) => return Err(String::from("failed to initialise consul http client")),
        };
        Ok(Client { address: address, http: client })
    }

    fn get_response(&self, key: &str) -> Result<reqwest::Response, errors::Error> {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        url.set_query(Some("raw=true"));
        let response = self.http.get(url)?.send()?;
        if !response.status().is_success() {
            return Err(errors::Error::Empty)
        }
        Ok(response)
    }
}

impl Source for Client {
    fn get_string(&self, key: &str) -> Result<String, errors::Error> {
        let mut string = String::new();
        self.get_response(key)?.read_to_string(&mut string)?;
        Ok(string)
    }

    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        Ok(self.get_response(key)?.json()?)
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
