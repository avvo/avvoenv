use std;
use std::io::Write;

use avvoenv;

extern crate getopts;

pub fn parse_opts(opts: &getopts::Options, args: Vec<String>, format_help: &Fn() -> String) -> getopts::Matches {
    match opts.parse(args) {
        Ok(ref matches) if matches.opt_present("v") => {
            println!("{} {}", avvoenv::NAME, avvoenv::VERSION);
            std::process::exit(0);
        }
        Ok(ref matches) if matches.opt_present("h") => {
            println!("{}", format_help());
            std::process::exit(0);
        }
        Ok(matches) => matches,
        Err(f) => { 
            println!("{}", f);
            warnln!("{}", format_help());
            std::process::exit(1);
        }
    }
}
