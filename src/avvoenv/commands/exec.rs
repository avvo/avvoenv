use std;
use std::os::unix::process::CommandExt;

use avvoenv::commands::helpers;
use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub struct Exec;

impl Command for Exec {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} exec [options] <command>", program)
    }

    fn add_opts(&self, mut opts: getopts::Options) -> getopts::Options {
        opts = helpers::add_fetch_opts(opts);
        opts.optflag("F", "force", "ignore errors and always execute <command>");
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
        let ignore_errors = matches.opt_present("F");
        match helpers::env_from_opts(matches) {
            Ok(env) => {
                command.envs(env.vars().iter());
                ErrorWithMessage(format!("{}", command.exec()))
            }
            Err(_) if ignore_errors => ErrorWithMessage(format!("{}", command.exec())),
            Err(res) => res,
        }
    }
}
