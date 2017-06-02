use std::result::Result;

use avvoenv::errors;
use avvoenv::source::Source;

extern crate hyper;
extern crate serde;
extern crate serde_json;

pub struct Client {
    address: hyper::Url,
    http: hyper::Client,
}

impl Client {
    pub fn new(mut address: hyper::Url, prefix: String) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv")
            .push(&prefix);
        Ok(Client { address: address, http: hyper::Client::new() })
    }
}

impl Source for Client {
    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
    where T: serde::de::DeserializeOwned {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        url.set_query(Some("raw=true"));
        let response = self.http.get(url).send()?;
        if response.status != hyper::Ok {
            return Err(errors::Error::Empty)
        }
        Ok(serde_json::from_reader(response)?)
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
