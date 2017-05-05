use std;
use std::os::unix::process::CommandExt;
use std::io::Write;

pub fn plugin(subcommand: String, args: Vec<String>) -> ! {
    let mut command = std::process::Command::new(format!("{}-{}", ::avvoenv::NAME, subcommand));
    command.args(args);
    warnln!("error executing {}-{}: {}", ::avvoenv::NAME, subcommand, command.exec());
    std::process::exit(1);
}
