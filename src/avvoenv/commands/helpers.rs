use std;
use std::collections::HashMap;
use std::io::Read;

use avvoenv;
use avvoenv::Env;
use avvoenv::commands;
use avvoenv::commands::CommandResult::*;
use avvoenv::rancher_metadata;
use avvoenv::source::vault;
use avvoenv::source::consul;

extern crate getopts;
extern crate reqwest;
extern crate serde_yaml;
extern crate rpassword;
extern crate serde;
extern crate shlex;
extern crate glob;

static CONSUL_HTTP_ADDR: &'static str = "http://127.0.0.1:8500";
static VAULT_ADDR: &'static str = "https://127.0.0.1:8200";

pub fn add_fetch_opts(mut opts: getopts::Options) -> getopts::Options {
    opts.optopt("s", "service", "set the service name", "NAME");
    opts.optopt("c", "consul", "set the consul host", "URL");
    opts.optopt("u", "vault", "set the vault host", "URL");
    opts.optflagopt("", "dev", "authenticate with vault", "USER");
    opts.optmulti("a", "add", "add an environment variable", "KEY=VALUE");
    opts.optmulti("i", "include", "filter fetched variables", "PATTERN");
    opts.optmulti("e", "exclude", "filter fetched variables", "PATTERN");
    opts.optopt("t", "vault-token", "set the vault token", "TOKEN");
    opts.optopt("r", "app-user", "authenticate with vault app-user", "VAULT_APP_USER");
    opts.optopt("p", "app-id", "authenticate with vault app-id", "VAULT_APP_ID");
    opts.optopt("k", "kubernetes-role", "authenticate with vault role", "KUBERNETES_ROLE");
    opts.optflag("", "no-rancher-metadata", "skip variables from Rancher metadata");
    opts
}

pub fn env_from_opts(matches: &getopts::Matches) -> Result<Env, commands::CommandResult> {
    let service = match guess_service(matches) {
        Some(val) => val,
        None => return Err(ErrorWithHelpMessage(String::from("Required option 'service' missing.")))
    };
    let consul_url = match opt_host(matches, "consul", "CONSUL_HTTP_ADDR", CONSUL_HTTP_ADDR) {
        Ok(val) => val,
        Err(s) => return Err(ErrorWithMessage(s)),
    };
    let consul_client = match consul::Client::new(consul_url) {
        Ok(val) => val,
        Err(e) => return Err(ErrorWithMessage(format!("{}", e))),
    };
    let vault_url = match opt_host(matches, "vault", "VAULT_ADDR", VAULT_ADDR) {
        Ok(val) => val,
        Err(s) => return Err(ErrorWithMessage(s)),
    };
    let mut vault_client = match vault::Client::new(vault_url) {
        Ok(val) => val,
        Err(e) => return Err(ErrorWithMessage(format!("{}", e))),
    };
    if matches.opt_present("dev") {
        let username = match opt_env(matches, "dev", "USER") {
            Some(val) => val,
            None => return Err(ErrorWithMessage(String::from("Could not determine dev user"))),
        };
        let password = rpassword::prompt_password_stderr(&format!("Password for {}:", username)).expect("couldn't get password");
        if vault_client.ldap_auth(username, password).is_err() {
            return Err(ErrorWithMessage(String::from("Authentication failed")));
        };
    } else if let Some(app_id) = opt_env(matches, "app-id", "VAULT_APP_ID") {
        let app_user = match opt_env(matches, "app-user", "VAULT_APP_USER") {
            Some(val) => val,
            None => return Err(ErrorWithMessage(String::from("Could not determine app-user"))),
        };
        if vault_client.app_id_auth(app_id, app_user).is_err() {
            return Err(ErrorWithMessage(String::from("Authentication failed")));
        };
    } else if let Some(role) = opt_env(matches, "kubernetes-role", "KUBERNETES_ROLE") {
        if vault_client.kubernetes_auth(role).is_err() {
            return Err(ErrorWithMessage(String::from("Authentication failed")))
        }
    } else {
        let mut path = std::env::home_dir().unwrap_or(std::path::PathBuf::from("/"));
        path.push(".vault-token");
        match opt_env_file(matches, "vault-token", "VAULT_TOKEN", &path) {
            Some(val) => vault_client.token = Some(val),
            None => return Err(ErrorWithHelpMessage(String::from("Required option 'vault-token' missing."))),
        };
        let _ = vault_client.renew_token();
    };

    let include = match patterns(matches.opt_strs("include")) {
        Ok(v) => v,
        Err(_) => return Err(ErrorWithMessage(String::from("Invalid include pattern."))),
    };
    let exclude = match patterns(matches.opt_strs("exclude")) {
        Ok(v) => v,
        Err(_) => return Err(ErrorWithMessage(String::from("Invalid exclude pattern."))),
    };

    let add = matches.opt_strs("add");
    let extra: HashMap<String, String> = add.iter()
        .filter_map(|s| shlex::split(s))
        .flat_map(|v| v)
        .map(|pair| {
            let mut parts = pair.splitn(2, "=");
            (parts.next().unwrap().to_string(), parts.next().unwrap_or("").to_string())
        }).collect();

    let rancher_client = if matches.opt_present("no-rancher-metadata") {
        None
    } else {
        let url_string = "http://rancher-metadata";
        let rancher_url = match reqwest::Url::parse(url_string) {
            Ok(val) => val,
            Err(e) => return Err(ErrorWithMessage(format!("{} for {:?}", e, url_string))),
        };
        match rancher_metadata::Client::new(rancher_url) {
            Ok(val) => Some(val),
            Err(e) => return Err(ErrorWithMessage(format!("{}", e))),
        }
    };

    match avvoenv::Env::fetch(service, consul_client, vault_client, rancher_client, include, exclude, extra) {
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

fn patterns(list: Vec<String>) -> Result<Vec<glob::Pattern>, glob::PatternError> {
    let strings: Vec<String> = list.iter()
        .filter_map(|s| shlex::split(s))
        .flat_map(|v| v)
        .flat_map(|s| {
            let v: Vec<String> = s.split(",").map(String::from).collect();
            v
        }).collect();
    let mut patterns = Vec::with_capacity(strings.len());
    for s in strings {
        patterns.push(glob::Pattern::new(&s)?);
    }
    Ok(patterns)
}
