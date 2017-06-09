# avvoenv

avvoenv fetches the environment variables for Avvo services.

See [the man page](avvoenv.1.ronn) for more details.

## Developing

avvoenv is written in Rust, you can install Rust with:

    curl https://sh.rustup.rs -sSf | sh

You can build and run avvoenv in debug mode with:

    cargo run -- <avvoenv args>

And build for release with:

    cargo build --release

There is a Rakefile with tasks for building, building the man page, and
installing locally: `rake build`, `rake man`, and `rake install`.
