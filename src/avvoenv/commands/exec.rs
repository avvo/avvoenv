use std;
use std::os::unix::process::CommandExt;
use std::io::Write;
use std::error::Error;

use avvoenv;
use avvoenv::commands::helpers;

extern crate getopts;

pub fn exec(program: String, mut opts: getopts::Options, args: Vec<String>) -> ! {
    opts.optflag("i", "isolate", "ignore the inherited env when executing <command>");

    let format_help = || {
        let brief = format!("Usage: {} [options] <command>", program);
        return opts.usage(&brief);
    };
    let matches = helpers::parse_opts(&opts, args, &format_help);

    let mut command = match matches.free.get(0) {
        Some(ref s) => std::process::Command::new(s),
        None => {
            warnln!("{}", format_help());
            std::process::exit(1);
        }
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
        }
        Err(e) => {
            warnln!("{}", e.description());
            std::process::exit(1);
        }
    };
    warnln!("{}", command.exec());
    std::process::exit(1);
}
