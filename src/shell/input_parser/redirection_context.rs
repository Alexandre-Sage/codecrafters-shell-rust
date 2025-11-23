use std::path::PathBuf;

use crate::exceptions::commands::CommandError;

#[derive(Debug, PartialEq)]
pub enum RedirectionChannel {
    Stdout,
    Stderr,
}

#[derive(Debug, PartialEq)]
pub enum RedirectionType {
    WriteOutput(RedirectionChannel),
}

impl TryFrom<&str> for RedirectionType {
    type Error = CommandError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ">" | "1>" => Ok(Self::WriteOutput(RedirectionChannel::Stdout)),
            "2>" => Ok(Self::WriteOutput(RedirectionChannel::Stderr)),
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

    pub fn should_write_stderr(&self) -> bool {
        matches!(
            self.redirection_type,
            RedirectionType::WriteOutput(RedirectionChannel::Stderr)
        )
    }

    pub fn should_write_stdout(&self) -> bool {
        matches!(
            self.redirection_type,
            RedirectionType::WriteOutput(RedirectionChannel::Stdout)
        )
    }
}
