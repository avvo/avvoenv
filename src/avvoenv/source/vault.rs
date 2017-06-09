use std;
use std::result::Result;
use std::io::Read;

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

#[derive(Serialize)]
pub struct AuthRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub client_token: String,
}

#[derive(Deserialize)]
pub struct AuthResponseWrapper {
    pub auth: AuthResponse,
}

#[derive(Deserialize)]
pub struct LeaderResponse {
    pub ha_enabled: bool,
    pub is_self: bool,
    pub leader_address: String,
}

pub struct Client {
    address: hyper::Url,
    pub token: Option<String>,
    http: hyper::Client,
}

impl Client {
    pub fn new(mut address: hyper::Url) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1");
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
        Ok(Client { address: address, token: None, http: client })
    }

    #[must_use]
    pub fn ldap_auth(&mut self, username: String, password: String) -> Result<(), errors::Error> {
        // workaround Vault (0.5.2?) being janky and (ldap?) auth only working
        // against the leader
        self.resolve_leader()?;
        let request = AuthRequest { password: password };
        let response: AuthResponseWrapper = self.post_json(&format!("auth/ldap/login/{}", username), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    #[must_use]
    fn resolve_leader(&mut self) -> Result<(), errors::Error> {
        let info: LeaderResponse = self.get_json("/sys/leader")?;
        if info.ha_enabled && !info.is_self {
            self.address = hyper::Url::parse(&info.leader_address)
                .expect("invalid leader address returned from vault");
        }
        Ok(())
    }

    fn get_json<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        Ok(serde_json::from_str(&self.get_string(key)?)?)
    }

    fn post_json<S, D>(&self, key: &str, data: &S) -> Result<D, errors::Error>
        where S: serde::ser::Serialize,
        D: serde::de::DeserializeOwned {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        let mut headers = hyper::header::Headers::new();
        if self.token.is_some() {
            headers.set(VaultToken(self.token.as_ref().unwrap().clone()));
        };
        let response = self.http
            .post(url)
            .headers(headers)
            .body(&serde_json::to_string(data).unwrap())
            .send()?;
        if response.status != hyper::Ok {
            return Err(errors::Error::Empty)
        }
        Ok(serde_json::from_reader(response)?)
    }
}

impl Source for Client {
    fn get_string(&self, key: &str) -> Result<String, errors::Error> {
        let mut url = self.address.clone();
        url.path_segments_mut().expect("invalid base URL").push(key);
        let mut headers = hyper::header::Headers::new();
        if self.token.is_some() {
            headers.set(VaultToken(self.token.as_ref().unwrap().clone()));
        };
        let mut response = self.http.get(url).headers(headers).send()?;
        if response.status != hyper::Ok {
            return Err(errors::Error::Empty)
        }
        let mut string = String::new();
        response.read_to_string(&mut string)?;
        Ok(string)
    }

    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        let result: serde_json::value::Value = self.get_json(key)?;
        match result.get("data") {
            Some(val) => Ok(serde_json::from_value(val.clone()).expect("failed to upcast json value")),
            None => Err(errors::Error::Empty),
        }
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
