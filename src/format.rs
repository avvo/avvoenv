use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fmt,
    io::{self, Write},
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
pub enum Format {
    Env,
    Defaults,
    Hocon,
    Json,
    Properties,
    Yaml,
}

impl Format {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Format {
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("defaults") | Some("sh") => Format::Defaults,
            Some("hocon") => Format::Hocon,
            Some("js") | Some("json") => Format::Json,
            Some("properties") => Format::Properties,
            Some("yml") | Some("yaml") => Format::Yaml,
            _ => Format::Env,
        }
    }

    pub fn to_writer<W: Write>(
        &self,
        writer: W,
        env: HashMap<String, String>,
    ) -> Result<(), FormatError> {
        match self {
            Format::Env => write_env(writer, env),
            Format::Defaults => write_defaults(writer, env),
            Format::Hocon => write_hocon(writer, env),
            Format::Json => write_json(writer, env),
            Format::Properties => write_properties(writer, env),
            Format::Yaml => write_yaml(writer, env),
        }
    }
}

#[derive(Debug)]
pub struct ParseFormatError(String);

impl fmt::Display for ParseFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown format {:?}", self.0)
    }
}

impl Error for ParseFormatError {}

impl FromStr for Format {
    type Err = ParseFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "env" => Ok(Format::Env),
            "defaults" => Ok(Format::Defaults),
            "hcon" => Ok(Format::Hocon),
            "json" => Ok(Format::Json),
            "properties" => Ok(Format::Properties),
            "yaml" => Ok(Format::Yaml),
            _ => Err(ParseFormatError(s.to_owned())),
        }
    }
}

#[derive(Debug)]
pub enum FormatError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    YamlError(serde_yaml::Error),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FormatError::IoError(e) => e.fmt(f),
            FormatError::JsonError(e) => e.fmt(f),
            FormatError::YamlError(e) => e.fmt(f),
        }
    }
}

impl Error for FormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FormatError::IoError(e) => Some(e),
            FormatError::JsonError(e) => Some(e),
            FormatError::YamlError(e) => Some(e),
        }
    }
}

impl From<io::Error> for FormatError {
    fn from(e: io::Error) -> FormatError {
        FormatError::IoError(e)
    }
}

impl From<serde_json::Error> for FormatError {
    fn from(e: serde_json::Error) -> FormatError {
        FormatError::JsonError(e)
    }
}

impl From<serde_yaml::Error> for FormatError {
    fn from(e: serde_yaml::Error) -> FormatError {
        FormatError::YamlError(e)
    }
}

fn write_env<W: Write>(mut writer: W, env: HashMap<String, String>) -> Result<(), FormatError> {
    for (key, val) in env {
        writeln!(writer, "{}={}", key, val)?;
    }
    Ok(())
}

fn write_defaults<W: Write>(
    mut writer: W,
    env: HashMap<String, String>,
) -> Result<(), FormatError> {
    for (key, val) in env {
        writeln!(
            writer,
            "export {}={}",
            key,
            shell_escape::escape(val.into())
        )?;
    }
    Ok(())
}

fn write_yaml<W: Write>(writer: W, env: HashMap<String, String>) -> Result<(), FormatError> {
    serde_yaml::to_writer(writer, &env)?;
    Ok(())
}

fn write_json<W: Write>(writer: W, env: HashMap<String, String>) -> Result<(), FormatError> {
    serde_json::to_writer_pretty(writer, &env)?;
    Ok(())
}

fn write_hocon<W: Write>(mut writer: W, env: HashMap<String, String>) -> Result<(), FormatError> {
    for (key, val) in env {
        write!(writer, "{} : ", key)?;
        serde_json::to_writer(&mut writer, &val)?;
        writeln!(writer)?;
    }
    Ok(())
}

fn write_properties<W: Write>(
    mut writer: W,
    env: HashMap<String, String>,
) -> Result<(), FormatError> {
    for (key, val) in env {
        for c in key.chars() {
            match c {
                '\\' => write!(writer, "\\\\")?,
                '\n' => write!(writer, "\\n")?,
                '\r' => write!(writer, "\\r")?,
                '\t' => write!(writer, "\\t")?,
                ' ' => write!(writer, "\\ ")?,
                _ => write!(writer, "{}", c)?,
            }
        }
        write!(writer, " = ")?;
        for c in val.chars() {
            match c {
                '\\' => write!(writer, "\\\\")?,
                '\n' => write!(writer, "\\n")?,
                '\r' => write!(writer, "\\r")?,
                '\t' => write!(writer, "\\t")?,
                _ => write!(writer, "{}", c)?,
            }
        }
        writeln!(writer)?;
    }
    Ok(())
}
