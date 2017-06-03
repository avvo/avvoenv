use std;
use std::os::unix::process::CommandExt;

use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub struct Plugin;

impl Command for Plugin {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} <command> [options]", program)
    }

    fn call(&self, matches: getopts::Matches) -> CommandResult {
        let plugin = match matches.free.get(0) {
            Some(s) => s,
            None => return ErrorWithHelp,
        };
        let name = format!("{}-{}", ::avvoenv::NAME, plugin);
        let mut command = std::process::Command::new(&name);
        command.args(matches.free[1..].iter());
        ErrorWithMessage(format!("error executing {}: {}", name, command.exec()))
    }
}
