use std;
use std::os::unix::process::CommandExt;
use std::io::Write;

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
    command.env("FOO", "bar");
    warnln!("{}", command.exec());
    std::process::exit(1);
}
