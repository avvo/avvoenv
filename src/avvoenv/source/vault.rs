use std;
use std::result::Result;
use std::io::Read;

use avvoenv::errors;
use avvoenv::source::Source;

extern crate reqwest;
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

#[derive(Serialize)]
pub struct AuthAppIdRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct AuthKubernetesRequest {
    pub jwt: String,
    pub role: String
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
    address: reqwest::Url,
    pub token: Option<String>,
    http: reqwest::Client,
}

#[derive(Serialize)]
pub struct TokenRenewRequest {
}

impl Client {
    pub fn new(mut address: reqwest::Url) -> Result<Client, String> {
        if address.cannot_be_a_base() {
            return Err(format!("{:?} is not a valid base URL", address));
        };
        address.path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("");
        let client = match reqwest::Client::new() {
            Ok(value) => value,
            Err(_) => return Err(String::from("failed to initialise vault http client")),
        };
        Ok(Client { address: address, token: None, http: client })
    }

    // #[must_use]
    pub fn ldap_auth(&mut self, username: String, password: String) -> Result<(), errors::Error> {
        // workaround Vault (0.5.2?) being janky and (ldap?) auth only working
        // against the leader
        self.resolve_leader()?;
        let request = AuthRequest { password: password };
        let response: AuthResponseWrapper = self.post_json(&format!("auth/ldap/login/{}", username), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    pub fn app_id_auth(&mut self, app_id: String, user_id: String) -> Result<(), errors::Error> {
        let request = AuthAppIdRequest { user_id };
        let response: AuthResponseWrapper = self.post_json(&format!("auth/app-id/login/{}", app_id), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    pub fn kubernetes_auth(&mut self, service_name: String) -> Result<(), errors::Error> {
        let mut file = std::fs::File::open("/var/run/secrets/kubernetes.io/serviceaccount/token")?;
        let mut jwt = String::new();
        file.read_to_string(&mut jwt)?;
        let request = AuthKubernetesRequest { jwt, role: service_name };
        let response: AuthResponseWrapper = self.post_json(&format!("auth/kubernetes/login"), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    pub fn renew_token(&mut self) -> Result<(), errors::Error> {
        let _:AuthResponseWrapper = self.post_json("/auth/token/renew-self", &TokenRenewRequest {})?;
        Ok(())
    }

    // #[must_use]
    fn resolve_leader(&mut self) -> Result<(), errors::Error> {
        let info: LeaderResponse = self.get_response("/sys/leader")?.json()?;
        if info.ha_enabled && !info.is_self {
            let mut leader_address = reqwest::Url::parse(&info.leader_address)
                .expect("invalid leader address returned from vault");
            leader_address.path_segments_mut()
                .expect("invalid base URL")
                .push("v1")
                .push("");
            self.address = leader_address;
        }
        Ok(())
    }

    fn post_json<S, D>(&self, key: &str, data: &S) -> Result<D, errors::Error>
        where S: serde::ser::Serialize,
        D: serde::de::DeserializeOwned {
        let url = self.address.join(key.trim_left_matches(|c| c == '/'))?;
        let mut request = self.http.post(url)?;
        request.json(data)?;
        if self.token.is_some() {
            request.header(VaultToken(self.token.as_ref().unwrap().clone()));
        };
        let mut response = request.send()?;
        if !response.status().is_success() {
            return Err(errors::Error::Empty)
        }
        Ok(response.json()?)
    }

    fn get_response(&self, key: &str) -> Result<reqwest::Response, errors::Error> {
        let url = self.address.join(key.trim_left_matches(|c| c == '/'))?;
        let mut request = self.http.get(url)?;
        if self.token.is_some() {
            request.header(VaultToken(self.token.as_ref().unwrap().clone()));
        };
        let response = request.send()?;
        if !response.status().is_success() {
            return Err(errors::Error::Empty)
        }
        Ok(response)
    }
}

impl Source for Client {
    fn get_string(&self, key: &str) -> Result<String, errors::Error> {
        let mut string = String::new();
        self.get_response(key)?.read_to_string(&mut string)?;
        Ok(string)
    }

    fn get<T>(&self, key: &str) -> Result<T, errors::Error>
        where T: serde::de::DeserializeOwned {
        let result: serde_json::value::Value = self.get_response(key)?.json()?;
        match result.get("data") {
            Some(val) => Ok(serde_json::from_value(val.clone()).expect("failed to upcast json value")),
            None => Err(errors::Error::Empty),
        }
    }

    fn address(&self) -> &str {
        self.address.as_str()
    }
}
