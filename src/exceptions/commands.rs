#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum CommandError {
    #[error("{0}: command not found")]
    CommandNotFound(String),
}
