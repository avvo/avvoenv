use std::{fmt, iter::IntoIterator, thread::sleep, time::Duration, vec};

use serde::Deserialize;
use reqwest::Url;

pub struct Client {
    address: Url,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct Container {
    primary_ip: String,
}

#[derive(Deserialize)]
struct Host {
    labels: Labels,
}

#[derive(Deserialize)]
struct Labels {
    fqdn: String,
}

#[derive(Deserialize)]
pub struct Info {
    container: Container,
    host: Host,
}

impl IntoIterator for Info {
    type Item = (String, String);
    type IntoIter = vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            (String::from("RANCHER_IP"), self.container.primary_ip),
            (String::from("STATSD_HOST"), self.host.labels.fqdn),
        ]
        .into_iter()
    }
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
    pub fn new() -> Client {
        Client {
            address: "http://rancher-metadata/2015-12-19/".parse().unwrap(),
            http: reqwest::Client::new(),
        }
    }

    fn get<T>(&self, path: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let url = self.address.join(path.trim_left_matches(|c| c == '/'))?;
        let request = self
            .http
            .get(url)
            .header(reqwest::header::ACCEPT, "application/json");
        let mut response = request.send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Error);
        }
        Ok(response.json()?)
    }

    fn get_retry<T>(&self, path: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let mut tries = 0;
        loop {
            match self.get(path) {
                Ok(v) => return Ok(v),
                Err(e) => {
                    tries += 1;
                    if tries > 5 {
                        return Err(e.into());
                    }
                    sleep(Duration::from_secs(tries));
                }
            }
        }
    }

    pub fn info(&self) -> Result<Option<Info>, Error> {
        self.get_retry("self")
    }
}
