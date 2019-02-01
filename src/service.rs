use std::fmt;

#[derive(Debug)]
struct NotUnicode(std::ffi::OsString);

impl std::error::Error for NotUnicode {}

impl fmt::Display for NotUnicode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not valid unicode: {:?}", self.0)
    }
}

#[derive(Debug)]
struct NoneError;

impl std::error::Error for NoneError {}

impl fmt::Display for NoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value missing")
    }
}

pub(crate) fn name(service: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let mut service = service;
    if service.is_none() {
        match std::fs::File::open("requirements.yml") {
            Ok(f) => {
                let buf = std::io::BufReader::new(f);
                let reqs: serde_yaml::Value = serde_yaml::from_reader(buf)?;
                if let Some(value) = reqs.get("service_name") {
                    service = Some(serde_yaml::to_string(value)?);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => Err(e)?,
        };
    };
    if service.is_none() {
        let dir = std::env::current_dir()?;
        if let Some(os_str) = dir.file_name() {
            if let Some(opt_s) = os_str.to_str() {
                service = Some(opt_s.to_owned());
            } else {
                Err(NotUnicode(os_str.to_owned()))?
            }
        }
    };
    service
        .map(|s| s.replace('_', "-").to_lowercase())
        .ok_or_else(|| NoneError.into())
}
