use std;
use std::os::unix::process::CommandExt;

use avvoenv;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;
extern crate hyper;

pub static BRIEF: &'static str = "Usage: {} exec [options] <command>";
pub static CONSUL_HTTP_ADDR: &'static str = "http://127.0.0.1:8500";
pub static VAULT_ADDR: &'static str = "https://127.0.0.1:8200";

pub fn add_opts(mut opts: getopts::Options) -> getopts::Options {
    opts.reqopt("s", "service", "set the service name", "NAME");
    opts.optopt("c", "consul", "set the consul host", "URL");
    opts.optopt("u", "vault", "set the vault host", "URL");
    opts.optopt("t", "vault-token", "set the vault token", "TOKEN");
    opts.optflag("i", "isolate", "ignore the inherited env when executing <command>");
    opts
}

pub fn call(matches: getopts::Matches) -> CommandResult {
    let service = matches.opt_str("service").expect("required argument not supplied");
    let consul_url = match opt_host(&matches, "consul", "CONSUL_HTTP_ADDR", CONSUL_HTTP_ADDR) {
        Ok(val) => val,
        Err(s) => return ErrorWithMessage(s),
    };
    let vault_url = match opt_host(&matches, "vault", "VAULT_ADDR", VAULT_ADDR) {
        Ok(val) => val,
        Err(s) => return ErrorWithMessage(s),
    };
    let vault_token = match opt_env(&matches, "vault-token", "VAULT_TOKEN") {
        Some(val) => val,
        None => return ErrorWithHelpMessage(String::from("Required option 'vault-token' missing.")),
    };
    let mut command = match matches.free.get(0) {
        Some(ref s) => std::process::Command::new(s),
        None => return ErrorWithHelp,
    };
    command.args(matches.free[1..].iter());
    if matches.opt_present("i") {
        command.env_clear();
    }

    match avvoenv::Env::fetch(service, consul_url, vault_url, vault_token) {
        Ok(env) => {
            // switch to command.envs(env.vars()) when that's stable
            for (key, val) in env.vars() {
                command.env(key, val);
            }
            ErrorWithMessage(format!("{}", command.exec()))
        }
        Err(e) => ErrorWithMessage(format!("{}", e)),
    }
}

fn opt_env(matches: &getopts::Matches, name: &str, var: &str) -> Option<String> {
    matches.opt_str(name).or_else(|| std::env::var(var).ok())
}

fn opt_env_default(matches: &getopts::Matches, name: &str, var: &str, default: &str) -> String {
    opt_env(matches, name, var).unwrap_or(String::from(default))
}

fn opt_host(matches: &getopts::Matches, name: &str, var: &str, default: &str) -> Result<hyper::Url, String> {
    let url_string = opt_env_default(matches, name, var, default);
    match hyper::Url::parse(&url_string) {
        Ok(url) => Ok(url),
        Err(e) => Err(format!("{} for {:?}", e, url_string)),
    }
}
