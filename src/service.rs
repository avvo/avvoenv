use std::{ffi, fmt, io};

use log::{debug, trace};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    NoneError,
    NotUnicode(ffi::OsString),
    YamlError(serde_yaml::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::NoneError => write!(f, "service name missing"),
            Error::NotUnicode(s) => write!(f, "service name valid unicode: {:?}", s),
            Error::YamlError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            Error::NoneError | Error::NotUnicode(_) => None,
            Error::YamlError(e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Error {
        Error::YamlError(e)
    }
}

pub(crate) fn name(service: Option<String>) -> Result<String, Error> {
    let mut service = service;
    if service.is_none() {
        trace!("Trying service from requirements.yml");
        match std::fs::File::open("requirements.yml") {
            Ok(f) => {
                let buf = std::io::BufReader::new(f);
                let reqs: serde_yaml::Value = serde_yaml::from_reader(buf)?;
                if let Some(serde_yaml::Value::String(val)) = reqs.get("service_name") {
                    debug!("Got service name {:?} from requirements.yml", val);
                    service = Some(val.clone());
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        };
    };
    if service.is_none() {
        let dir = std::env::current_dir()?;
        if let Some(os_str) = dir.file_name() {
            if let Some(opt_s) = os_str.to_str() {
                debug!("Got service name {:?} from current dir", opt_s);
                service = Some(opt_s.to_owned());
            } else {
                return Err(Error::NotUnicode(os_str.to_owned()))
            }
        }
    };
    service
        .map(|s| s.replace('_', "-").to_lowercase())
        .ok_or_else(|| Error::NoneError)
}
