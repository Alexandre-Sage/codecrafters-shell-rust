use std::str::FromStr;

use crate::exceptions::commands::CommandError;

pub(crate) mod builtins;
pub mod registry;

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) enum CommandToken {
    Exit,
    Echo,
    Type,
}

impl FromStr for CommandToken {
    type Err = CommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            _ => Err(CommandError::CommandNotFound(s.to_owned())),
        }
    }
}
