use crate::exceptions::commands::CommandError;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum ApplicationError {
    #[error(transparent)]
    CommandError(#[from] CommandError),
}
