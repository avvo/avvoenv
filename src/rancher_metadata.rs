use std::{fmt, iter::IntoIterator, net::ToSocketAddrs, thread::sleep, time::Duration};

use log::trace;
use reqwest::Url;
use serde::Deserialize;

use crate::client_error::ClientError;

pub fn is_available() -> bool {
    "rancher-metadata:80".to_socket_addrs().is_ok()
}

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
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            rancher_ip: Some(self.container.primary_ip),
            statsd_host: Some(self.host.labels.fqdn),
        }
    }
}

pub struct IntoIter {
    rancher_ip: Option<String>,
    statsd_host: Option<String>,
}

impl Iterator for IntoIter {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.rancher_ip.is_some() {
            self.rancher_ip
                .take()
                .map(|val| (String::from("RANCHER_IP"), val))
        } else {
            self.statsd_host
                .take()
                .map(|val| (String::from("STATSD_HOST"), val))
        }
    }
}

#[derive(Debug)]
pub struct Error(ClientError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl<T: Into<ClientError>> From<T> for Error {
    fn from(e: T) -> Error {
        Error(e.into())
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
        let url = self.address.join(path.trim_start_matches(|c| c == '/'))?;
        let request = self
            .http
            .get(url.clone())
            .header(reqwest::header::ACCEPT, "application/json");
        trace!("{:?}", request);
        let mut response = request
            .send()
            .map_err(|e| ClientError::with_url(url.clone(), e))?;
        trace!("{:?}", response);
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(ClientError::ServerError(response).into());
        }
        Ok(response.json().map_err(|e| ClientError::with_url(url, e))?)
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
