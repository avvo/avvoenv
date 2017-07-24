use std;
use std::collections::HashMap;
use std::io::Read;

use avvoenv;
use avvoenv::Env;
use avvoenv::commands;
use avvoenv::commands::CommandResult::*;
use avvoenv::source::vault;
use avvoenv::source::consul;

extern crate getopts;
extern crate reqwest;
extern crate serde_yaml;
extern crate rpassword;
extern crate serde;
extern crate shlex;

static CONSUL_HTTP_ADDR: &'static str = "http://127.0.0.1:8500";
static VAULT_ADDR: &'static str = "https://127.0.0.1:8200";

pub fn add_fetch_opts(mut opts: getopts::Options) -> getopts::Options {
    opts.optopt("s", "service", "set the service name", "NAME");
    opts.optopt("c", "consul", "set the consul host", "URL");
    opts.optopt("u", "vault", "set the vault host", "URL");
    opts.optflagopt("", "dev", "authenticate with vault", "USER");
    opts.optmulti("a", "add", "add an environment variable", "KEY=VALUE");
    opts.optopt("t", "vault-token", "set the vault token", "TOKEN");
    opts
}

pub fn env_from_opts(matches: getopts::Matches) -> Result<Env, commands::CommandResult> {
    let service = match guess_service(&matches) {
        Some(val) => val,
        None => return Err(ErrorWithHelpMessage(String::from("Required option 'service' missing.")))
    };
    let consul_url = match opt_host(&matches, "consul", "CONSUL_HTTP_ADDR", CONSUL_HTTP_ADDR) {
        Ok(val) => val,
        Err(s) => return Err(ErrorWithMessage(s)),
    };
    let consul_client = match consul::Client::new(consul_url) {
        Ok(val) => val,
        Err(e) => return Err(ErrorWithMessage(format!("{}", e))),
    };
    let vault_url = match opt_host(&matches, "vault", "VAULT_ADDR", VAULT_ADDR) {
        Ok(val) => val,
        Err(s) => return Err(ErrorWithMessage(s)),
    };
    let mut vault_client = match vault::Client::new(vault_url) {
        Ok(val) => val,
        Err(e) => return Err(ErrorWithMessage(format!("{}", e))),
    };
    if matches.opt_present("dev") {
        let username = match opt_env(&matches, "dev", "USER") {
            Some(val) => val,
            None => return Err(ErrorWithMessage(String::from("Could not determine dev user"))),
        };
        let password = rpassword::prompt_password_stderr(&format!("Password for {}:", username)).expect("couldn't get password");
        if vault_client.ldap_auth(username, password).is_err() {
            return Err(ErrorWithMessage(String::from("Authentication failed")));
        };
    } else {
        let mut path = std::env::home_dir().unwrap_or(std::path::PathBuf::from("/"));
        path.push(".vault-token");
        match opt_env_file(&matches, "vault-token", "VAULT_TOKEN", &path) {
            Some(val) => vault_client.token = Some(val),
            None => return Err(ErrorWithHelpMessage(String::from("Required option 'vault-token' missing."))),
        };
    };
    let add = matches.opt_strs("add");
    let extra: HashMap<String, String> = add.iter()
        .map(|s| shlex::split(s))
        .filter(|o| o.is_some())
        .map(|o| o.unwrap())
        .flat_map(|v| v)
        .map(|pair| {
            let mut parts = pair.splitn(2, "=");
            (parts.next().unwrap().to_string(), parts.next().unwrap_or("").to_string())
        }).collect();

    match avvoenv::Env::fetch(service, consul_client, vault_client, extra) {
        Ok(env) => Ok(env),
        Err(e) => Err(ErrorWithMessage(format!("{}", e))),
    }
}

fn opt_env(matches: &getopts::Matches, name: &str, var: &str) -> Option<String> {
    matches.opt_str(name).or_else(|| std::env::var(var).ok())
}

fn opt_env_default(matches: &getopts::Matches, name: &str, var: &str, default: &str) -> String {
    opt_env(matches, name, var).unwrap_or(String::from(default))
}

fn opt_env_file(matches: &getopts::Matches, name: &str, var: &str, path: &std::path::Path) -> Option<String> {
    opt_env(matches, name, var)
        .or_else(|| {
            std::fs::File::open(path)
                .ok()
                .map(std::io::BufReader::new)
                .and_then(|mut buf| {
                    let mut string = String::new();
                    if buf.read_to_string(&mut string).is_ok() {
                        Some(string.trim_right().to_string())
                    } else {
                        None
                    }
                })
        })
}

fn opt_host(matches: &getopts::Matches, name: &str, var: &str, default: &str) -> Result<reqwest::Url, String> {
    let url_string = opt_env_default(matches, name, var, default);
    match reqwest::Url::parse(&url_string) {
        Ok(url) => Ok(url),
        Err(e) => Err(format!("{} for {:?}", e, url_string)),
    }
}

pub fn guess_service(matches: &getopts::Matches) -> Option<String> {
    service_from_options(matches)
        .or(service_from_env())
        .or(service_from_requirements_yml())
        .or(service_from_current_dir())
}

fn service_from_options(matches: &getopts::Matches) -> Option<String> {
    matches.opt_str("service").map(|s| format_service_name(&s))
}

fn service_from_env() -> Option<String> {
    std::env::var("SERVICE").ok().map(|s| format_service_name(&s))
}

fn service_from_requirements_yml() -> Option<String> {
    std::fs::File::open("requirements.yml")
        .ok()
        .map(|file| std::io::BufReader::new(file))
        .and_then(|buf| serde_yaml::from_reader(buf).ok() as Option<serde_yaml::Value>)
        .and_then(|yml| yml.get("service_name")
            .and_then(serde_yaml::Value::as_str)
            .map(format_service_name))
}

fn service_from_current_dir() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|q| q.to_owned()))
        .and_then(|p| p.to_str().map(format_service_name))
}

fn format_service_name(input: &str) -> String {
    input.replace("_", "-").to_lowercase()
}
