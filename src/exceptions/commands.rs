#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum CommandError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
    #[error("Too many arguments: expected {0}, got {1}")]
    TooManyArguments(String, usize),
    #[error("Invalid arg type expected: {0}")]
    ParsingError(String),
    #[error(transparent)]
    TypeCommandError(#[from] super::type_command_error::TypeCommandError),
    #[error("No args received expected at least: {0}")]
    EmptyArgs(usize),
}
