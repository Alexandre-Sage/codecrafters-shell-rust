use crate::exceptions::commands::ShellError;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandResult {
    Exit(i32),
    Stdio(String, String),
    Empty,
    Error(ShellError),
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
    fn execute(&self, args: &[String]) -> Result<CommandResult, ShellError>;
}
