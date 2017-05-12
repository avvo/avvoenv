use std::io::Write;

mod avvoenv;
use avvoenv::commands::*;
use avvoenv::commands::CommandResult;
use avvoenv::commands::CommandResult::*;

extern crate getopts;
use getopts::{Options,Matches};

#[macro_use]
extern crate serde_derive;

// like println, but to stderr
macro_rules! warnln(
    ($($arg:tt)*) => { {
        writeln!(&mut ::std::io::stderr(), $($arg)*).unwrap();
    } }
);

fn main() {
    // set basic options
    let mut opts = Options::new();
    opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
    opts.optflag("h", "help", "print this message");
    opts.optflag("v", "version", "print the version");

    // gather the arguments and seperate out the program name
    let mut args: Vec<String> = std::env::args().collect();
    let program = args.remove(0);

    // get the function that handles the subcommand the user specified
    let (brief, add_opts, call):
        (&str, fn(Options) -> Options, fn(Matches) -> CommandResult) =
        match args.get(0).map(String::as_ref) {
            Some("exec") => {
                args.remove(0); // remove the command name
                (exec::BRIEF, exec::add_opts, exec::call)
            },
            Some(_) if !args[0].starts_with("-") => (plugin::BRIEF, plugin::add_opts, plugin::call),
            _ => (default::BRIEF, default::add_opts, default::call),
    };

    // let the command add extra options
    opts = add_opts(opts);

    // handle errors, -v, -h, and call the command
    let result = match opts.parse(args) {
        Err(f) => ErrorWithHelpMessage(format!("{}", f)),
        Ok(ref matches) if matches.opt_present("v") => {
            let version = format!("{} {}", avvoenv::NAME, avvoenv::VERSION);
            SuccessWithMessage(version)
        }
        Ok(ref matches) if matches.opt_present("h") => SuccessWithHelp,
        Ok(matches) => call(matches),
    };

    // figure out what output and output it
    match result {
        Success => (),
        SuccessWithMessage(ref msg) | ErrorWithMessage(ref msg) => {
            println!("{}", msg);
        }
        SuccessWithHelp | ErrorWithHelp => {
            warnln!("{}", opts.usage(&brief.replace("{}", &program)));
        }
        ErrorWithHelpMessage(ref msg) => {
            let b = brief.replace("{}", &program);
            let m = format!("{}\n\n{}", msg, b);
            warnln!("{}", opts.usage(&m));
        }
    }

    // figure out how to exit
    match result {
        Success | SuccessWithMessage(_) | SuccessWithHelp => std::process::exit(0),
        ErrorWithMessage(_) | ErrorWithHelpMessage(_) | ErrorWithHelp => std::process::exit(1),
    }
}
