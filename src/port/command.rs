use crate::exceptions::commands::CommandError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandResult {
    Exit(i32),
    Message(String),
    Stdio(String, String),
}

pub(crate) trait Command {
    fn execute(&self, args: &str) -> Result<CommandResult, CommandError>;
}
