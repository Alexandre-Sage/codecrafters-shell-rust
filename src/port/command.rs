use crate::exceptions::commands::CommandError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandResult {
    Exit(i32),
    Message(String),
    Stdio(String, String),
    Empty,
}

pub(crate) trait Command {
    fn execute(&self, args: &[String]) -> Result<CommandResult, CommandError>;
}
