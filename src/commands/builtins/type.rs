use std::str::FromStr;

use crate::{
    commands::CommandToken,
    exceptions::{
        commands::CommandError::{self, TooManyArguments},
        type_command_error::TypeCommandNotFound,
    },
    port::command::{Command, CommandResult},
};

pub struct Type;

impl Command for Type {
    fn execute(
        &self,
        args: &str,
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        if args.is_empty() {
            return Err(CommandError::EmptyArgs(1));
        }

        let args_parts = args.split_whitespace().collect::<Vec<&str>>();

        if args_parts.len() > 1 {
            return Err(TooManyArguments("1".to_string(), args_parts.len()));
        }

        match CommandToken::from_str(args) {
            Ok(_) => Ok(CommandResult::Message(
                args.to_owned() + " is a shell builtin",
            )),
            Err(_) => Err(TypeCommandNotFound(args.to_owned()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        exceptions::{commands::CommandError, type_command_error::TypeCommandNotFound},
        port::command::CommandResult,
    };

    use super::*;

    #[test]
    fn type_echo_builtin() {
        let result = Type.execute("echo");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("echo is a shell builtin".to_string())
        )
    }

    #[test]
    fn type_itself_is_builtin() {
        let result = Type.execute("type");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("type is a shell builtin".to_string())
        )
    }

    #[test]
    fn type_unknown_command() {
        let result = Type.execute("nonexistentcommand");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TypeCommandNotFound(TypeCommandNotFound(
                "nonexistentcommand".to_string()
            ))
        )
    }

    #[test]
    fn type_empty_args() {
        let result = Type.execute("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::EmptyArgs(1))
    }

    #[test]
    fn type_too_many_args() {
        let result = Type.execute("echo exit");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TooManyArguments("1".to_string(), 2)
        )
    }

    #[test]
    fn type_multiple_args() {
        let result = Type.execute("echo exit ls");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TooManyArguments("1".to_string(), 3)
        )
    }
}
