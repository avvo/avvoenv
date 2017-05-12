use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub static BRIEF: &'static str = "Usage: {} <command> [options]\n\n   exec   Run a command";

pub fn add_opts(opts: getopts::Options) -> getopts::Options {
    opts
}

pub fn call(_: getopts::Matches) -> CommandResult {
    ErrorWithHelp
}
