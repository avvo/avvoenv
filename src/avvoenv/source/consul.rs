use std::result::Result;
use std::io::Read;

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
    pub fn new(mut address: hyper::Url) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv");
        Ok(Client { address: address, http: hyper::Client::new() })
    }
}

impl Source for Client {
    fn get_string(&self, key: &str) -> Result<String, errors::Error> {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        url.set_query(Some("raw=true"));
        let mut response = self.http.get(url).send()?;
        if response.status != hyper::Ok {
            return Err(errors::Error::Empty)
        }
        let mut string = String::new();
        response.read_to_string(&mut string)?;
        Ok(string)
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
