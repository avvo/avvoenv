use std;
use std::os::unix::process::CommandExt;

use avvoenv;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub static BRIEF: &'static str = "Usage: {} exec [options] <command>";

pub fn add_opts(mut opts: getopts::Options) -> getopts::Options {
    opts.optflag("i", "isolate", "ignore the inherited env when executing <command>");
    opts
}

pub fn call(matches: getopts::Matches) -> CommandResult {
    let mut command = match matches.free.get(0) {
        Some(ref s) => std::process::Command::new(s),
        None => return ErrorWithHelp,
    };
    command.args(matches.free[1..].iter());
    if matches.opt_present("i") {
        command.env_clear();
    }
    match avvoenv::Env::fetch("http://127.0.0.1:8500") {
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
