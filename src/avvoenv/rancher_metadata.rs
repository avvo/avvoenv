use std;

use avvoenv::errors;

extern crate reqwest;
extern crate serde;
extern crate serde_json;

pub struct Client {
    address: reqwest::Url,
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

impl std::iter::IntoIterator for Info {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            (String::from("RANCHER_IP"), self.container.primary_ip),
            (String::from("STATSD_HOST"), self.host.labels.fqdn),
        ].into_iter()
    }
}

impl Client {
    pub fn new(mut address: reqwest::Url) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("2015-12-19")
            .push("");
        let client = match reqwest::Client::new() {
            Ok(value) => value,
            Err(_) => return Err(String::from("failed to initialise rancher metadata http client")),
        };
        Ok(Client { address: address, http: client })
    }

    fn get_response(&self, path: &str) -> Result<reqwest::Response, errors::Error> {
        let url = self.address.join(path.trim_left_matches(|c| c == '/'))?;
        let mut request = self.http.get(url)?;
        request.header(reqwest::header::Accept::json());
        let response = request.send()?;
        if !response.status().is_success() {
            return Err(errors::Error::Empty)
        }
        Ok(response)
    }

    fn get<T>(&self, path: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        Ok(self.get_response(path)?.json()?)
    }

    pub fn info(&self) -> Result<Info, errors::Error> {
        self.get("self")
    }
}
