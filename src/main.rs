mod client_error;
mod consul;
mod env;
mod format;
mod prompt;
mod rancher_metadata;
mod service;
mod vault;

use std::{
    cmp::max,
    fs::File,
    io,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
};

use glob::Pattern;
use log::{debug, error, info, trace, warn};
use reqwest::Url;
use structopt::{
    clap::AppSettings::{
        ArgRequiredElseHelp, ArgsNegateSubcommands, DisableHelpSubcommand, TrailingVarArg,
        VersionlessSubcommands,
    },
    StructOpt,
};

use format::Format;

fn main() {
    let opts = Opts::from_args();

    let verbosity = std::env::var("AVVOENV_LOG_LEVEL")
        .map(verbosity)
        .unwrap_or(0);
    stderrlog::new()
        .module(module_path!())
        .quiet(opts.quiet)
        .verbosity(max(verbosity, opts.verbosity))
        .init()
        .unwrap();

    debug!("{:#?}", opts);

    let result = match opts.subcommand {
        Some(Subcommand::Exec(opts)) => exec(opts),
        Some(Subcommand::Write(opts)) => write(opts),
        Some(Subcommand::Service(opts)) => service(opts),
        None => plugin(opts.script.unwrap(), opts.args),
    };

    if let Err(e) = result {
        debug!("{:?}", e);
        error!("{}", e);
        std::process::exit(1);
    }
}

fn verbosity(s: String) -> usize {
    match s.parse() {
        Ok(log::Level::Error) => 0,
        Ok(log::Level::Warn) => 1,
        Ok(log::Level::Info) => 2,
        Ok(log::Level::Debug) => 3,
        Ok(log::Level::Trace) => 4,
        Err(_) => s.parse().unwrap_or(0),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    settings = &[ArgsNegateSubcommands, ArgRequiredElseHelp, DisableHelpSubcommand, VersionlessSubcommands]
)]
struct Opts {
    /// Verbose mode, multiples increase the verbosity
    #[structopt(short = "v", long = "verbose", parse(from_occurrences), global = true)]
    verbosity: usize,
    /// Silence output
    #[structopt(short = "q", long = "quiet", global = true, conflicts_with = "verbose")]
    quiet: bool,
    #[structopt(subcommand)]
    subcommand: Option<Subcommand>,
    #[structopt(empty_values = false, hidden = true)]
    script: Option<String>,
    #[structopt(hidden = true)]
    args: Vec<String>,
}

#[derive(StructOpt, Debug)]
enum Subcommand {
    /// Execute the given command with the fetched environment variables
    #[structopt(
        name = "exec",
        no_version,
        setting = TrailingVarArg
    )]
    Exec(ExecOpts),
    /// Write the fetched environment variables to a file
    #[structopt(name = "write", no_version)]
    Write(WriteOpts),
    /// Print the canonical name of the current service
    #[structopt(name = "service", no_version)]
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
        parse(from_str = parse_add)
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
        required_unless_one = &["dev", "app_user", "app_id"]
    )]
    token: Option<vault::Secret>,
    /// authenticate with vault app-user
    #[structopt(
        short = "r",
        long = "app-user",
        value_name = "VAULT_APP_USER",
        requires = "app_id",
        conflicts_with = "dev",
        env = "VAULT_APP_USER"
    )]
    app_user: Option<vault::Secret>,
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
    /// [env: NO_RANCHER_METADATA=]
    #[structopt(long = "no-rancher-metadata")]
    skip_rancher_metadata: bool,
}

fn parse_add(s: &str) -> (String, String) {
    let mut parts = s.splitn(2, '=');
    (
        parts.next().unwrap().to_string(),
        parts.next().unwrap_or("").to_string(),
    )
}

#[derive(StructOpt, Debug)]
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
    trace!("Running exec subcommand");

    if opts.cmd.is_empty() {
        info!("Required argument CMD was not provided");
        ExecOpts::clap().write_help(&mut std::io::stderr()).unwrap();
        std::process::exit(1);
    }

    let mut command = std::process::Command::new(&opts.cmd[0]);
    command.args(&opts.cmd[1..]);

    if opts.isolate {
        debug!("Clearing inherited system environment due to --isolate option");
        command.env_clear();
    }

    match env::fetch(opts.fetch) {
        Ok(env) => {
            trace!("Got env: {:#?}", env);
            command.envs(env);
        }
        Err(ref e) if opts.force => {
            debug!("{:?}", e);
            warn!("{}", e);
            debug!("Ignoring error due to --force option");
        }
        Err(e) => return Err(e.into()),
    };

    debug!("Executing {:?}", &opts.cmd[0]);
    trace!("Args: {:?}", &opts.cmd[1..]);
    Err(Box::new(command.exec()))
}

#[derive(StructOpt, Debug)]
struct WriteOpts {
    #[structopt(flatten)]
    fetch: FetchOpts,
    /// set the output format
    #[structopt(
        short = "f",
        long = "format",
        value_name = "FORMAT",
        possible_values = &["env", "defaults", "hcon", "json", "properties", "yaml"]
    )]
    format: Option<Format>,
    /// File to write
    #[structopt(name = "FILE")]
    path: PathBuf,
}

fn write(opts: WriteOpts) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Running write subcommand");

    let path = opts.path;
    let format = opts.format.unwrap_or_else(|| Format::from_path(&path));
    debug!("Using format {:?}", format);
    let env = env::fetch(opts.fetch)?;
    trace!("Got env: {:#?}", env);
    if path == Path::new("-") {
        trace!("Writing to stdout");
        format.to_writer(io::stdout(), env)?;
    } else {
        trace!("Writing to {:?}", path);
        format.to_writer(File::create(path)?, env)?;
    };
    Ok(())
}

#[derive(StructOpt, Debug)]
struct ServiceOpts {
    /// set the service name
    #[structopt(short = "s", long = "service", value_name = "NAME", env = "SERVICE")]
    service: Option<String>,
}

fn service(opts: ServiceOpts) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Running service subcommand");

    let service = service::name(opts.service)?;
    println!("{}", service);
    Ok(())
}

fn plugin(name: String, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Running plugin");

    let name = format!(concat!(env!("CARGO_PKG_NAME"), "-{}"), name);
    let mut command = std::process::Command::new(&name);
    command.args(&args);
    debug!("Executing {:?}", name);
    trace!("Args: {:?}", args);
    Err(Box::new(command.exec()))
}
