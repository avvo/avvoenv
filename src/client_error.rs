use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ClientError {
    BaseUrlError(reqwest::Url),
    JsonError {
        url: reqwest::Url,
        source: serde_json::Error,
    },
    RequestError {
        url: reqwest::Url,
        source: reqwest::Error,
    },
    ServerError(reqwest::blocking::Response),
    UrlError(url::ParseError),
}

impl ClientError {
    pub fn with_url<T>(url: reqwest::Url, detail: T) -> ClientError
    where
        (reqwest::Url, T): Into<ClientError>,
    {
        (url, detail).into()
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::BaseUrlError(url) => write!(f, "Cannot be a Base: {:?}", url),
            ClientError::JsonError { url, source } => write!(f, "{}: {}", url, source),
            ClientError::RequestError { ref source, .. } if source.url().is_some() => source.fmt(f),
            ClientError::RequestError { url, source } => write!(f, "{}: {}", url, source),
            ClientError::ServerError(response) => write!(f, "{:?}", response),
            ClientError::UrlError(e) => e.fmt(f),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::BaseUrlError(_) | ClientError::ServerError(_) => None,
            ClientError::JsonError { source, .. } => Some(source),
            ClientError::RequestError { source, .. } => Some(source),
            ClientError::UrlError(e) => Some(e),
        }
    }
}

impl From<url::ParseError> for ClientError {
    fn from(e: url::ParseError) -> ClientError {
        ClientError::UrlError(e)
    }
}

impl From<(reqwest::Url, reqwest::Error)> for ClientError {
    fn from((url, source): (reqwest::Url, reqwest::Error)) -> ClientError {
        ClientError::RequestError { url, source }
    }
}

impl From<(reqwest::Url, serde_json::Error)> for ClientError {
    fn from((url, source): (reqwest::Url, serde_json::Error)) -> ClientError {
        ClientError::JsonError { url, source }
    }
}

impl From<reqwest::blocking::Response> for ClientError {
    fn from(response: reqwest::blocking::Response) -> ClientError {
        ClientError::ServerError(response)
    }
}
