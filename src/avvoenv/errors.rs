use std;
use std::error::Error as StdError;
use std::fmt;

extern crate hyper;
extern crate serde_json;

#[derive(Debug)]
pub enum Error {
    ParseError(hyper::error::ParseError),
    HttpError(hyper::Error),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    Empty,
    BadVersion,
}

impl From<hyper::error::ParseError> for Error {
    fn from(err: hyper::error::ParseError) -> Error {
        Error::ParseError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::HttpError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError(ref err) => err.description(),
            Error::HttpError(ref err) => err.description(),
            Error::JsonError(ref err) => err.description(),
            Error::IoError(ref err) => err.description(),
            Error::Empty => "empty",
            Error::BadVersion => "bad version",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::ParseError(ref err) => Some(err as &StdError),
            Error::HttpError(ref err) => Some(err as &StdError),
            Error::JsonError(ref err) => Some(err as &StdError),
            Error::IoError(ref err) => Some(err as &StdError),
            Error::Empty => None,
            Error::BadVersion => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref err) => fmt::Display::fmt(err, f),
            Error::HttpError(ref err) => fmt::Display::fmt(err, f),
            Error::JsonError(ref err) => fmt::Display::fmt(err, f),
            Error::IoError(ref err) => fmt::Display::fmt(err, f),
            Error::Empty => write!(f, "empty"),
            Error::BadVersion => write!(f, "bad version"),
        }
    }
}
