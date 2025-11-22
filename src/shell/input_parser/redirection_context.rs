use std::{i8, path::PathBuf, process::Stdio};

use crate::{exceptions::commands::CommandError, shell::input_parser::commons::REDIRECT_OUTPUT};

#[derive(Debug, PartialEq)]
pub enum RedirectionChannel {
    Stdout,
}

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    Output(RedirectionChannel),
}

impl TryFrom<&str> for RedirectionType {
    type Error = CommandError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ">" | "1>" => Ok(Self::Output(RedirectionChannel::Stdout)),
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
