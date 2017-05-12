use std;
use std::os::unix::process::CommandExt;

use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub static BRIEF: &'static str = "Usage: {} <command> [options]";

pub fn add_opts(opts: getopts::Options) -> getopts::Options {
    opts
}

pub fn call(matches: getopts::Matches) -> CommandResult {
    let plugin = match matches.free.get(0) {
        Some(s) => s,
        None => return ErrorWithHelp,
    };
    let name = format!("{}-{}", ::avvoenv::NAME, plugin);
    let mut command = std::process::Command::new(&name);
    command.args(matches.free[1..].iter());
    ErrorWithMessage(format!("error executing {}: {}", name, command.exec()))
}
