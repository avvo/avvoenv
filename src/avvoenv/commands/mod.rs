pub enum CommandResult {
    #[allow(dead_code)]
    Success,
    SuccessWithMessage(String),
    SuccessWithHelp,
    ErrorWithMessage(String),
    ErrorWithHelpMessage(String),
    ErrorWithHelp,
}

mod exec;
pub use self::exec::Exec;
mod default;
pub use self::default::Default;
mod plugin;
pub use self::plugin::Plugin;
mod command;
pub use self::command::Command;
mod helpers;