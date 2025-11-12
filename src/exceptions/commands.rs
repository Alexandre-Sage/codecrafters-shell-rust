#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum CommandError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
    #[error("Too much args provided expected: {0} received: {1}")]
    TooMuchArgs(String, usize),
    #[error("Invalid arg type expected: {0}")]
    ParsingError(String),
}
