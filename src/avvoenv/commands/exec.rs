use std;
use std::os::unix::process::CommandExt;

use avvoenv::commands::helpers;
use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;
extern crate hyper;

pub struct Exec;

impl Command for Exec {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} exec [options] <command>", program)
    }

    fn add_opts(&self, mut opts: getopts::Options) -> getopts::Options {
        opts = helpers::add_fetch_opts(opts);
        opts.optflag("i", "isolate", "ignore the inherited env when executing <command>");
        opts
    }

    fn call(&self, matches: getopts::Matches) -> CommandResult {
        let mut command = match matches.free.get(0) {
            Some(ref s) => std::process::Command::new(s),
            None => return ErrorWithHelp,
        };
        command.args(matches.free[1..].iter());
        if matches.opt_present("i") {
            command.env_clear();
        }
        let env = match helpers::env_from_opts(matches) {
            Ok(val) => val,
            Err(res) => return res,
        };
        // switch to command.envs(env.vars().iter()) when that's stable
        for (key, val) in env.vars().iter() {
            command.env(key, val);
        }
        ErrorWithMessage(format!("{}", command.exec()))
    }
}
