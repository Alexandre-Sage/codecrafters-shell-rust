#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{0}: not found")]
pub(crate) struct TypeCommandNotFound(pub String);
