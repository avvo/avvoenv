use avvoenv::commands::helpers;
use avvoenv::commands::Command;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;

pub struct Service;

impl Command for Service {
    fn brief(&self, program: &str) -> String {
        format!("Usage: {} service [options]", program)
    }

    fn add_opts(&self, mut opts: getopts::Options) -> getopts::Options {
        opts.optopt("s", "service", "set the service name", "NAME");
        opts
    }

    fn call(&self, matches: getopts::Matches) -> CommandResult {
        if matches.free.len() > 0 {
            return ErrorWithHelp;
        }
        match helpers::guess_service(&matches) {
            Some(val) => SuccessWithMessage(val),
            None => ErrorWithMessage(String::from("service could not be determined"))
        }
        
    }
}
