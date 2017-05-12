pub mod commands;
pub mod source;
mod env;
pub use self::env::Env;
pub mod errors;

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
