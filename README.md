# avvoenv

avvoenv loads config directly from Vault and Consul so that you can just
restart your application to start using new config.

See [the man page](avvoenv.1.ronn) for more details.

## Build

avvoenv is written in [Rust] 2018 Edition, using Rust 1.32. You can install
Rust using [rustup]. [Cargo] is used to build avvoenv and manage dependencies.
If you're new to Rust, [The Rust Programming Language][book] - an introductory
book about Rust - is available free online.

[Rust]: https://www.rust-lang.org/
[Actix Web]: https://actix.rs
[rustup]: https://www.rust-lang.org/en-US/install.html
[Cargo]: https://doc.rust-lang.org/stable/cargo/
[book]: https://doc.rust-lang.org/book/2018-edition/index.html

avvoenv can then be built for development with:

    cargo build

The binary will be written to `target/debug/avvoenv`.

Building for release can be done with:

    cargo build --release

The binary will be written to `target/release/avvoenv`.

The `./package` script automates building for macOS (assuming it's the host)
and Linux, and then zip/tar.gz-ing the binaries ready for distribution. The
packages will be written to `target/package/avvoenv-mac.zip` and
`target/x86_64-unknown-linux-musl/package/avvoenv-linux.tar.gz`.

## Docs

The man page is generated with [ronn].

[ronn]: https://github.com/rtomayko/ronn

    gem install ronn
    ronn --roff avvoenv.1.ronn

Additionally, documentation can be generated from the source. This will include,
and cross link to, documentation for all libraries used.

Build the docs with:

    cargo doc --document-private-items

The docs will be written to `target/doc/avvoenv`.

To build the docs and open them in your web browser run:

cargo doc --document-private-items --open

## Run

Once built, simply run the binary as `path/to/avvoenv`, or place it within
your `$PATH` and run `avvoenv`.

During development you can build and run avvoenv with:

    cargo run -- [options]

## Config

    USAGE:
        avvoenv [FLAGS]
        avvoenv exec [FLAGS] [OPTIONS] --consul <URL> --vault-token <TOKEN> --vault <URL> [--] [CMD]...
        avvoenv write [FLAGS] [OPTIONS] <FILE> --consul <URL> --vault-token <TOKEN> --vault <URL>
        avvoenv service [FLAGS] [OPTIONS]
        avvoenv <SUBCOMMAND>

    FLAGS:
            --dev                    authenticate with vault
        -F, --force                  ignore errors and always execute <command>
        -h, --help                   Prints help information
        -I, --isolate                ignore the inherited env when executing <command>
        -q, --quiet                  Silence output
            --no-rancher-metadata
        -V, --version                Prints version information
        -v, --verbose                Verbose mode, multiples increase the verbosity

    OPTIONS:
        -a, --add <KEY=VALUE>...           add an environment variable
        -p, --app-id <VAULT_APP_ID>        authenticate with vault app-id [env: VAULT_APP_ID=]
        -r, --app-user <VAULT_APP_USER>    authenticate with vault app-user [env: VAULT_APP_USER=]
        -c, --consul <URL>                 set the consul host [env: CONSUL_HTTP_ADDR=]
        -e, --exclude <PATTERN>...         filter fetched variables
        -f, --format <FORMAT>              set the output format [possible values: env, defaults, hcon, json, properties, yaml]
        -i, --include <PATTERN>...         filter fetched variables
        -s, --service <NAME>               set the service name [env: SERVICE=]
        -t, --vault-token <TOKEN>          set the vault token [env: VAULT_TOKEN=]
        -u, --vault <URL>                  set the vault host [env: VAULT_ADDR=]

    ARGS:
        <CMD>...    Command to exec
        <FILE>      File to write

    SUBCOMMANDS:
        exec       Execute the given command with the fetched environment variables
        service    Print the canonical name of the current service
        write      Write the fetched environment variables to a file

avvoenv can also be configured with a number of environment variables:

| Environment variable | Description
|----------------------|---
| AVVOENV_LOG_LEVEL    | Set the logging verbosity
| CONSUL_HTTP_ADDR     | Set the consul host
| SERVICE              | Set the service name
| USER                 | The default user for Vault LDAP auth
| VAULT_ADDR           | Set the vault host
| VAULT_APP_ID         | Set the App ID for App ID auth
| VAULT_APP_USER       | Set the App User for App ID auth
| VAULT_TOKEN          | Set the vault token

## Troubleshooting

Logging can be enabled with the `-v` flag to enable warnings, `-vv` for basic
logging, `-vvv` for debug logs and `-vvvv` for tracing logs. Alternately the
log level can be set with the `AVVOENV_LOG_LEVEL` environment variable, either
`error`, `warn`, `info`, `debug`, `trace` or an integer from 0 to 4 inclusive.
