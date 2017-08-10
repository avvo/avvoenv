use std;
use std::fs::File;
use std::io;
use std::io::Write as StdWrite;
use std::collections::HashMap;

use avvoenv::commands::helpers;
use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;
extern crate shell_escape;
extern crate serde_yaml;
extern crate serde_json;

pub struct Write;

enum FormatType {
    Env,
    Defaults,
    YAML,
    JSON,
    HOCON,
    Properties,
}

impl Command for Write {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} write [options] FILE", program)
    }

    fn add_opts(&self, mut opts: getopts::Options) -> getopts::Options {
        opts = helpers::add_fetch_opts(opts);
        opts.optopt("f", "format", "set the output format", "FORMAT");
        opts
    }

    fn call(&self, matches: getopts::Matches) -> CommandResult {
        if matches.free.len() != 1 {
            return ErrorWithHelp;
        }
        let path = matches.free.get(0).expect("couldn't get file argument").clone();
        let format = match matches.opt_str("format") {
            Some(val) => {
                match format_type(val) {
                    Ok(val) => val,
                    Err(e) => return ErrorWithMessage(format!("{}", e)),
                }
            }
            None => guess_format_type(&path),
        };

        let file: Box<io::Write> = if path == "-" {
            Box::new(io::stdout())
        } else {
            match File::create(path) {
                Ok(f) => Box::new(f),
                Err(e) => return ErrorWithMessage(format!("{}", e)),
            }
        };

        let env = match helpers::env_from_opts(&matches) {
            Ok(val) => val,
            Err(res) => return res,
        };

        let mut buf = io::BufWriter::new(file);

        match buf.write_all(format.format(env.vars()).as_bytes()) {
            Ok(_) => Success,
            Err(e) => return ErrorWithMessage(format!("{}", e)),
        }
    }
}

fn format_type(type_string: String) -> Result<FormatType, String> {
    match type_string.as_ref() {
        "env" => Ok(FormatType::Env),
        "defaults" => Ok(FormatType::Defaults),
        "yaml" => Ok(FormatType::YAML),
        "json" => Ok(FormatType::JSON),
        "hocon" => Ok(FormatType::HOCON),
        "properties" => Ok(FormatType::Properties),
        _ => Err(format!("{} is not a valid format", type_string)),
    }
}

fn guess_format_type(input_path: &str) -> FormatType {
    let path = std::path::Path::new(input_path);
    match path.extension().and_then(|p| p.to_str()) {
        Some("defaults") => FormatType::Defaults,
        Some("sh") => FormatType::Defaults,
        Some("yml") => FormatType::YAML,
        Some("yaml") => FormatType::YAML,
        Some("json") => FormatType::JSON,
        Some("js") => FormatType::JSON,
        Some("hocon") => FormatType::HOCON,
        Some("properties") => FormatType::Properties,
        _ => FormatType::Env,
    }
}

fn escape_properties_key(value: &str) -> String {
    let chars: Vec<_> = value.chars().map(|c| {
        match c {
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            ' ' => "\\ ".to_string(),
            _ => c.to_string(),
        }
    }).collect();
    chars.join("")
}

fn escape_properties_value(value: &str) -> String {
    let chars: Vec<_> = value.chars().map(|c| {
        match c {
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            _ => c.to_string(),
        }
    }).collect();
    chars.join("")
}

impl FormatType {
    fn format(&self, vars: &HashMap<String, String>) -> String {
        match self {
            &FormatType::Env => {
                let pairs: Vec<_> = vars
                    .iter()
                    .map(|(key, val)| format!("{}={}\n", key, val))
                    .collect();
                pairs.join("")
            }
            &FormatType::Defaults => {
                let pairs: Vec<_> = vars
                    .iter()
                    .map(|(key, val)| {
                        let escaped_val = shell_escape::escape(val.clone().into());
                        format!("export {}={}\n", key, escaped_val)
                    }).collect();
                pairs.join("")
            }
            &FormatType::YAML => {
                serde_yaml::to_string(&vars).unwrap()
            }
            &FormatType::JSON => {
                serde_json::to_string(&vars).unwrap()
            }
            &FormatType::HOCON => {
                let pairs: Vec<_> = vars
                    .iter()
                    .map(|(key, val)| {
                        let escaped_val = serde_json::to_string(&val).unwrap();
                        format!("{} : {}\n", key, escaped_val)
                    }).collect();
                pairs.join("")
            }
            &FormatType::Properties => {
                let pairs: Vec<_> = vars
                    .iter()
                    .map(|(key, val)| {
                        let escaped_key = escape_properties_key(key);
                        let escaped_val = escape_properties_value(val);
                        format!("{} = {}\n", escaped_key, escaped_val)
                    }).collect();
                pairs.join("")
            }
        }
    }
}
