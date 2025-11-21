use std::path::PathBuf;

use crate::{exceptions::commands::CommandError, shell::input_parser::commons::REDIRECT_OUTPUT};

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    Output,
}

impl TryFrom<char> for RedirectionType {
    type Error = CommandError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            REDIRECT_OUTPUT => Ok(Self::Output),
            _ => Err(CommandError::Unknown("No redirection".to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct RedirectionContext {
    pub path: PathBuf,
    pub redirection_type: RedirectionType,
}

impl RedirectionContext {
    pub fn new(path: PathBuf, redirection_type: RedirectionType) -> Self {
        Self {
            path,
            redirection_type,
        }
    }
}
