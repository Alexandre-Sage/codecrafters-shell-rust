use std::str::FromStr;

use strum::IntoEnumIterator;

use crate::exceptions::commands::ShellError;

pub(crate) mod builtins;
pub mod registry;

#[derive(Debug, Hash, PartialEq, Eq, strum::EnumIter)]
pub(crate) enum CommandToken {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl FromStr for CommandToken {
    type Err = ShellError;
    fn from_str(command: &str) -> Result<Self, Self::Err> {
        match command {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            "pwd" => Ok(Self::Pwd),
            "cd" => Ok(Self::Cd),
            _ => Err(ShellError::CommandNotFound(command.to_owned())),
        }
    }
}

impl ToString for CommandToken {
    fn to_string(&self) -> String {
        match self {
            CommandToken::Echo => "echo",
            CommandToken::Cd => "cd",
            CommandToken::Pwd => "pwd",
            CommandToken::Type => "type",
            CommandToken::Exit => "exit",
        }
        .to_owned()
    }
}

impl CommandToken {
    pub fn into_completion() -> Vec<String> {
        CommandToken::iter()
            .map(|token| token.to_string())
            .collect()
    }
}
