use std::fmt;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::client_error::ClientError;

pub struct Client {
    address: Url,
    token: Option<String>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct Response<T> {
    data: T,
}

#[derive(Deserialize)]
struct LeaderResponse {
    ha_enabled: bool,
    is_self: bool,
    leader_address: String,
}

#[derive(Serialize)]
struct LdapAuthRequest<'a> {
    password: &'a str,
}

#[derive(Serialize)]
struct AppIdAuthRequest<'a> {
    user_id: &'a str,
}

#[derive(Deserialize)]
struct AuthResponse {
    client_token: String,
}

#[derive(Deserialize)]
struct AuthResponseWrapper {
    auth: AuthResponse,
}

#[derive(Debug)]
pub struct Error(ClientError);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vault: {}", self.0)
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
    pub fn new(mut address: Url) -> Result<Client, Error> {
        if address.cannot_be_a_base() {
            return Err(ClientError::BaseUrlError(address).into());
        };
        address
            .path_segments_mut()
            .expect("invalid base URL")
            .push("v1")
            .push("");
        Ok(Client {
            address,
            token: None,
            http: reqwest::Client::new(),
        })
    }

    pub fn token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn ldap_auth(&mut self, username: &str, password: &str) -> Result<(), Error> {
        // workaround Vault (0.5.2?) being janky and (ldap?) auth only working
        // against the leader
        self.resolve_leader()?;
        let request = LdapAuthRequest { password };
        let response: AuthResponseWrapper =
            self.post(&format!("auth/ldap/login/{}", username), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    pub fn app_id_auth(&mut self, app_id: &str, user_id: &str) -> Result<(), Error> {
        let request = AppIdAuthRequest { user_id };
        let response: AuthResponseWrapper =
            self.post(&format!("auth/app-id/login/{}", app_id), &request)?;
        self.token = Some(response.auth.client_token);
        Ok(())
    }

    fn resolve_leader(&mut self) -> Result<(), Error> {
        let info = match self.get_internal::<LeaderResponse>("/sys/leader")? {
            Some(v) => v,
            None => return Ok(()),
        };
        if info.ha_enabled && !info.is_self {
            let mut leader_address = Url::parse(&info.leader_address)
                .expect("invalid leader address returned from vault");
            leader_address
                .path_segments_mut()
                .expect("invalid base URL")
                .push("v1")
                .push("");
            self.address = leader_address;
        }
        Ok(())
    }

    fn post<S, D>(&self, key: &str, data: &S) -> Result<D, Error>
    where
        S: serde::ser::Serialize,
        D: serde::de::DeserializeOwned,
    {
        let url = self.address.join(key.trim_left_matches(|c| c == '/'))?;
        let mut request = self.http.post(url).json(data);
        if let Some(ref token) = self.token {
            request = request.header("X-Vault-Token", token.as_str());
        };
        let mut response = request.send()?;
        if !response.status().is_success() {
            return Err(ClientError::ServerError(response).into());
        }
        Ok(response.json()?)
    }

    fn get_internal<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let url = self.address.join(key.trim_left_matches(|c| c == '/'))?;
        let mut request = self.http.get(url);
        if let Some(ref token) = self.token {
            request = request.header("X-Vault-Token", token.as_str());
        };
        let mut response = request.send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(ClientError::ServerError(response).into());
        }
        Ok(Some(response.json()?))
    }
}

impl crate::env::Client for Client {
    type Error = Error;

    fn get<T>(&self, key: &str) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        Ok(self.get_internal::<Response<T>>(key)?.map(|r| r.data))
    }
}
