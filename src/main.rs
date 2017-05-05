mod avvoenv;
use avvoenv::commands;

extern crate getopts;

fn main() {
    let mut opts = getopts::Options::new();
    opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree);
    opts.optflag("h", "help", "print this message");
    opts.optflag("v", "version", "print the version");

    let mut args: Vec<String> = std::env::args().collect();
    let program = args.remove(0);

    match args.get(0).map(String::as_ref) {
        Some("exec") => commands::exec(format!("{} {}", program, args.remove(0)), opts, args),
        Some(_) if !args[0].starts_with("-") => commands::plugin(args.remove(0), args),
        _ => commands::default(program, opts, args),
    }
}
