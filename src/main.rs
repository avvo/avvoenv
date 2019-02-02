mod client_error;
mod consul;
mod env;
mod prompt;
mod rancher_metadata;
mod service;
mod vault;

use std::os::unix::process::CommandExt;

use glob::Pattern;
use log::{debug, error};
use reqwest::Url;
use structopt::{
    clap::AppSettings::{
        ArgRequiredElseHelp, ArgsNegateSubcommands, DisableHelpSubcommand, TrailingVarArg,
        VersionlessSubcommands,
    },
    StructOpt,
};

fn main() {
    let opts = Opts::from_args();

    stderrlog::new()
        .quiet(opts.quiet)
        .verbosity(opts.verbosity)
        .init()
        .unwrap();

    debug!("{:?}", opts);

    let result = match opts.subcommand {
        Some(Subcommand::Exec(opts)) => exec(opts),
        Some(Subcommand::Write(opts)) => write(opts),
        Some(Subcommand::Service(opts)) => service(opts),
        None => plugin(opts.script.unwrap(), opts.args),
    };

    if let Err(e) = result {
        error!("{}", e);
        std::process::exit(1);
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    author = "",
    raw(
        version = "env!(\"CARGO_PKG_VERSION\")",
        settings = "&[ArgsNegateSubcommands, ArgRequiredElseHelp, DisableHelpSubcommand, TrailingVarArg, VersionlessSubcommands]"
    )
)]
struct Opts {
    /// Verbose mode, multiples increase the verbosity
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        raw(global = "true")
    )]
    verbosity: usize,
    /// Silence output
    #[structopt(
        short = "q",
        long = "quiet",
        raw(global = "true"),
        conflicts_with = "verbose"
    )]
    quiet: bool,
    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,
    #[structopt(raw(empty_values = "false", hidden = "true"))]
    script: Option<String>,
    #[structopt(raw(hidden = "true"))]
    args: Vec<String>,
}

#[derive(StructOpt, Debug)]
enum Subcommand {
    /// Execute the given command with the fetched environment variables
    #[structopt(name = "exec", author = "", version = "")]
    Exec(ExecOpts),
    /// Write the fetched environment variables to a file
    #[structopt(name = "write", author = "", version = "")]
    Write(WriteOpts),
    /// Print the canonical name of the current service
    #[structopt(name = "service", author = "", version = "")]
    Service(ServiceOpts),
}

#[derive(StructOpt, Debug)]
pub(crate) struct FetchOpts {
    /// set the service name
    #[structopt(short = "s", long = "service", value_name = "NAME", env = "SERVICE")]
    service: Option<String>,
    /// set the consul host
    #[structopt(
        short = "c",
        long = "consul",
        value_name = "URL",
        env = "CONSUL_HTTP_ADDR"
    )]
    consul: Url,
    /// set the vault host
    #[structopt(short = "u", long = "vault", value_name = "URL", env = "VAULT_ADDR")]
    vault: Url,
    /// authenticate with vault
    #[structopt(long = "dev")]
    dev: bool,
    /// add an environment variable
    #[structopt(
        short = "a",
        long = "add",
        value_name = "KEY=VALUE",
        parse(from_str = "parse_add")
    )]
    add: Vec<(String, String)>,
    /// filter fetched variables
    #[structopt(short = "i", long = "include", value_name = "PATTERN")]
    include: Vec<Pattern>,
    /// filter fetched variables
    #[structopt(short = "e", long = "exclude", value_name = "PATTERN")]
    exclude: Vec<Pattern>,
    /// set the vault token
    #[structopt(
        short = "t",
        long = "vault-token",
        value_name = "TOKEN",
        env = "VAULT_TOKEN",
        raw(required_unless_one = r#"&["dev", "app_user", "app_id"]"#)
    )]
    token: Option<String>,
    /// authenticate with vault app-user
    #[structopt(
        short = "r",
        long = "app-user",
        value_name = "VAULT_APP_USER",
        requires = "app_id",
        conflicts_with = "dev",
        env = "VAULT_APP_USER"
    )]
    app_user: Option<String>,
    /// authenticate with vault app-id
    #[structopt(
        short = "p",
        long = "app-id",
        value_name = "VAULT_APP_ID",
        requires = "app_user",
        conflicts_with = "dev",
        env = "VAULT_APP_ID"
    )]
    app_id: Option<String>,
}

fn parse_add(s: &str) -> (String, String) {
    let mut parts = s.splitn(2, "=");
    (
        parts.next().unwrap().to_string(),
        parts.next().unwrap_or("").to_string(),
    )
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "TrailingVarArg"))]
struct ExecOpts {
    #[structopt(flatten)]
    fetch: FetchOpts,
    /// ignore errors and always execute <command>
    #[structopt(short = "F", long = "force")]
    force: bool,
    /// ignore the inherited env when executing <command>
    #[structopt(short = "I", long = "isolate")]
    isolate: bool,
    /// Command to exec
    #[structopt(name = "CMD")]
    cmd: Vec<String>,
}

fn exec(opts: ExecOpts) -> Result<(), Box<dyn std::error::Error>> {
    if opts.cmd.len() < 1 {
        ExecOpts::clap().write_help(&mut std::io::stderr()).unwrap();
        std::process::exit(1);
    }

    let mut command = std::process::Command::new(&opts.cmd[0]);
    command.args(&opts.cmd[1..]);

    if opts.isolate {
        command.env_clear();
    }

    match env::fetch(opts.fetch) {
        Ok(env) => {
            command.envs(env);
        }
        Err(_) if opts.force => (),
        Err(e) => return Err(e.into()),
    };
    Err(Box::new(command.exec()))
}

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "TrailingVarArg"))]
struct WriteOpts {
    #[structopt(flatten)]
    fetch: FetchOpts,
    /// ignore errors and always execute <command>
    #[structopt(short = "f", long = "format", value_name = "FORMAT")]
    format: Option<String>,
    /// File to write
    #[structopt(name = "FILE")]
    file: String,
}

fn write(opts: WriteOpts) -> Result<(), Box<dyn std::error::Error>> {
    unimplemented!()
}

#[derive(StructOpt, Debug)]
struct ServiceOpts {
    /// set the service name
    #[structopt(short = "s", long = "service", value_name = "NAME", env = "SERVICE")]
    service: Option<String>,
}

fn service(opts: ServiceOpts) -> Result<(), Box<dyn std::error::Error>> {
    let service = service::name(opts.service)?;
    println!("{}", service);
    Ok(())
}

fn plugin(name: String, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let name = format!(concat!(env!("CARGO_PKG_NAME"), "-{}"), name);
    let mut command = std::process::Command::new(&name);
    command.args(args);
    Err(Box::new(command.exec()))
}
