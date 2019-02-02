use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ClientError {
    BaseUrlError(reqwest::Url),
    JsonError(serde_json::Error),
    RequestError(reqwest::Error),
    ServerError(reqwest::Response),
    UrlError(reqwest::UrlError),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::BaseUrlError(url) => write!(f, "Cannot be a Base: {:?}", url),
            ClientError::JsonError(e) => e.fmt(f),
            ClientError::RequestError(e) => e.fmt(f),
            ClientError::ServerError(response) => write!(f, "Bad response: {:?}", response),
            ClientError::UrlError(e) => e.fmt(f),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::BaseUrlError(_) | ClientError::ServerError(_) => None,
            ClientError::JsonError(e) => Some(e),
            ClientError::RequestError(e) => Some(e),
            ClientError::UrlError(e) => Some(e),
        }
    }
}

impl From<reqwest::UrlError> for ClientError {
    fn from(e: reqwest::UrlError) -> ClientError {
        ClientError::UrlError(e)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> ClientError {
        ClientError::RequestError(e)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> ClientError {
        ClientError::JsonError(e)
    }
}
