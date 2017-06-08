use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub struct Default;

impl Command for Default {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} <command> [options]

   exec      Run a command
   write     Write out to a file
   service   Print the service name", program)
    }

    fn call(&self, _: getopts::Matches) -> CommandResult {
        ErrorWithHelp
    }
}
