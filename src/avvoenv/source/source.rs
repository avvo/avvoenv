use std::result::Result;

use avvoenv::errors;

extern crate hyper;
extern crate serde;
extern crate serde_json;

pub trait Source {
    fn get_string(&self, key: &str) -> Result<String, errors::Error>;

    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        Ok(serde_json::from_str(&self.get_string(key)?)?)
    }

    fn address(&self) -> &str;
}
