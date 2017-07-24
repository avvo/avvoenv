use std::result::Result;

use avvoenv::errors;

extern crate serde;

pub trait Source {
    fn get_string(&self, key: &str) -> Result<String, errors::Error>;

    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned;

    fn address(&self) -> &str;
}
