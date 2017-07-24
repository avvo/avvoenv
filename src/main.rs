mod avvoenv;
use avvoenv::commands;
use avvoenv::commands::*;
use avvoenv::commands::CommandResult::*;

extern crate getopts;
use getopts::Options;

#[macro_use]
extern crate serde_derive;

// reqwest doesn't yet (re)export a macro for creating custom header types, so
// we need to directly depend on hyper for that for the moment
#[macro_use]
extern crate hyper;

fn main() {
    // set basic options
    let mut opts = Options::new();
    opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
    opts.optflag("h", "help", "print this message");
    opts.optflag("v", "version", "print the version");

    // gather the arguments and seperate out the program name
    let mut args: Vec<String> = std::env::args().collect();
    let program = args.remove(0);

    // get the object that handles the subcommand the user specified
    let command: Box<Command> = match args.get(0).map(String::as_ref) {
        Some("exec") => {
            args.remove(0); // remove the command name
            Box::new(commands::Exec)
        },
        Some("write") => {
            args.remove(0); // remove the command name
            Box::new(commands::Write)
        },
        Some("service") => {
            args.remove(0); // remove the command name
            Box::new(commands::Service)
        },
        Some(_) if !args[0].starts_with("-") => Box::new(commands::Plugin),
        _ => Box::new(commands::Default),
    };

    // let the command add extra options
    opts = command.add_opts(opts);

    // handle errors, -v, -h, and call the command
    let result = match opts.parse(args) {
        Err(f) => ErrorWithHelpMessage(format!("{}", f)),
        Ok(ref matches) if matches.opt_present("v") => {
            let version = format!("{} {}", avvoenv::NAME, avvoenv::VERSION);
            SuccessWithMessage(version)
        }
        Ok(ref matches) if matches.opt_present("h") => SuccessWithHelp,
        Ok(matches) => command.call(matches),
    };

    // figure out what output and output it
    match result {
        Success => (),
        SuccessWithMessage(ref msg) | ErrorWithMessage(ref msg) => {
            println!("{}", msg);
        }
        SuccessWithHelp | ErrorWithHelp => {
            eprintln!("{}", opts.usage(&command.brief(&program)));
        }
        ErrorWithHelpMessage(ref msg) => {
            let b = command.brief(&program);
            let m = format!("{}\n\n{}", msg, b);
            eprintln!("{}", opts.usage(&m));
        }
    }

    // figure out how to exit
    match result {
        Success | SuccessWithMessage(_) | SuccessWithHelp => std::process::exit(0),
        ErrorWithMessage(_) | ErrorWithHelpMessage(_) | ErrorWithHelp => std::process::exit(1),
    }
}
