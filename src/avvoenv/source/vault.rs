use std;
use std::result::Result;

use avvoenv::errors;
use avvoenv::source::Source;

extern crate hyper;
extern crate hyper_native_tls;
extern crate serde;
extern crate serde_json;

header! { (VaultToken, "X-Vault-Token") => [String] }

#[derive(Deserialize)]
struct Response {
    pub data: std::collections::HashMap<String, String>,
}

pub struct Client {
    address: hyper::Url,
    token: String,
    http: hyper::Client,
}

impl Client {
    pub fn new(mut address: hyper::Url, prefix: String, token: String) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("kv")
            .push(&prefix);
        let client = if address.scheme() == "https" {
            let ssl = match hyper_native_tls::NativeTlsClient::new() {
                Ok(val) => val,
                Err(e) => return Err(format!("{}", e)),
            };
            let connector = hyper::net::HttpsConnector::new(ssl);
            hyper::Client::with_connector(connector)
        } else {
            hyper::Client::new()
        };
        Ok(Client { address: address, token: token, http: client })
    }
}

impl Source for Client {
    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
    where T: serde::de::DeserializeOwned {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        let mut headers = hyper::header::Headers::new();
        headers.set(VaultToken(self.token.clone()));
        let response = self.http.get(url).headers(headers).send()?;
        if response.status != hyper::Ok {
            return Err(errors::Error::Empty)
        }
        let result: serde_json::value::Value = serde_json::from_reader(response)?;
        match result.get("data") {
            Some(val) => Ok(serde_json::from_value(val.clone()).expect("failed to upcast json value")),
            None => Err(errors::Error::Empty),
        }
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
