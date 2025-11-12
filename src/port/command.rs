use crate::exceptions::commands::CommandError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandResult {
    Exit(i32),
}

pub(crate) trait Command {
    fn execute(&self, args: &[&str]) -> Result<CommandResult, CommandError>;
}
