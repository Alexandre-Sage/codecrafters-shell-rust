use crate::exceptions::commands::CommandError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandResult {
    Exit(i32),
    Stdio(String, String),
    Empty,
    Error(CommandError),
}

impl CommandResult {
    pub fn stdout(buffer: impl Into<String>) -> Self {
        Self::Stdio(buffer.into(), String::new())
    }

    pub fn stderr(buffer: impl Into<String>) -> Self {
        Self::Stdio(String::new(), buffer.into())
    }
}

pub(crate) trait Command {
    fn execute(&self, args: &[String]) -> Result<CommandResult, CommandError>;
}
