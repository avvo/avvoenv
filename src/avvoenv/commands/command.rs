use avvoenv::commands::CommandResult;

extern crate getopts;

pub trait Command {
    fn brief(&self, program: &str) -> String;

    fn add_opts(&self, opts: getopts::Options) -> getopts::Options {
        opts
    }

    fn call(&self, matches: getopts::Matches) -> CommandResult;
}
