use std::result::Result;

use avvoenv::errors;

extern crate hyper;
extern crate serde;
extern crate serde_json;

pub struct Client {
    address: hyper::Url,
    http: hyper::Client,
}

impl Client {
    pub fn new(address: &str, prefix: &str) -> Result<Client, errors::Error> {
        let url = hyper::Url::parse(address)?.join("v1/kv/")?.join(prefix)?;
        Ok(Client { address: url, http: hyper::Client::new() })
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, errors::Error>
    where T: serde::de::DeserializeOwned {
        let mut url = self.address.join(key)?;
        url.set_query(Some("raw=true"));
        let response = self.http.get(url).send()?;
        if response.status != hyper::Ok {
            return Ok(None)
        }
        Ok(serde_json::from_reader(response)?)
    }
}
