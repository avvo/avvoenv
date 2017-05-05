use std;

use avvoenv::commands::helpers;

extern crate getopts;

pub fn default(program: String, opts: getopts::Options, args: Vec<String>) -> ! {
    let format_help = || {
        let commands = vec![
            "   exec   Run a command"
        ];
        let brief = format!("Usage: {} <command> [options]\n\n{}", program, commands.join("\n"));
        return opts.usage(&brief);
    };
    helpers::parse_opts(&opts, args, &format_help);

    println!("{}", format_help());
    std::process::exit(0);
}
