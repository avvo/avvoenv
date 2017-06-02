use std::result::Result;

use avvoenv::errors;

extern crate hyper;
extern crate serde;

pub trait Source {
    fn get<T>(&self, &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned;

    fn address(&self) -> &str;
}
