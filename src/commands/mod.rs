use std::str::FromStr;

use crate::exceptions::commands::CommandError;

pub(crate) mod builtins;
pub mod registry;

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) enum CommandToken {
    Exit,
    Echo,
    Type,
    Pwd,
}

impl FromStr for CommandToken {
    type Err = CommandError;
    fn from_str(command: &str) -> Result<Self, Self::Err> {
        match command {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            "pwd" => Ok(Self::Pwd),
            _ => Err(CommandError::CommandNotFound(command.to_owned())),
        }
    }
}
