#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TypeCommandError {
    #[error("{0}: not found")]
    NotFound(String),
}
