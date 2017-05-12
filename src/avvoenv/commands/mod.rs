pub enum CommandResult {
    #[allow(dead_code)]
    Success,
    SuccessWithMessage(String),
    SuccessWithHelp,
    ErrorWithMessage(String),
    ErrorWithHelpMessage(String),
    ErrorWithHelp,
}

pub mod exec;
pub mod default;
pub mod plugin;
