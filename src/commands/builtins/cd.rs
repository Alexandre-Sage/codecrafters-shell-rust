use std::{path::PathBuf, str::FromStr};

use crate::{
    exceptions::commands::CommandError,
    port::command::{Command, CommandResult},
};

pub struct Cd;

impl Command for Cd {
    fn execute(
        &self,
        args: &str,
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        let home_dir = std::env::home_dir().unwrap();

        if args.is_empty() {
            std::env::set_current_dir(home_dir)
                .map_err(|err| CommandError::Unknown(err.to_string()))?;

            return Ok(CommandResult::Empty);
        }

        let args_parts: Vec<_> = args.split_whitespace().collect();

        if args_parts.len() > 1 {
            return Err(CommandError::TooManyArguments(
                "1".to_string(),
                args_parts.len(),
            ));
        }

        let path = PathBuf::from_str(args.trim())
            .map_err(|_| CommandError::ParsingError("expected path like".to_string()))?;

        if !path.exists() {
            return Err(CommandError::DirectoryNotFound(path));
        }

        if !path.is_dir() {
            return Err(CommandError::NotADirectory(path));
        }

        std::env::set_current_dir(path).map_err(|err| CommandError::Unknown(err.to_string()))?;

        Ok(CommandResult::Empty)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_to_home_if_empty() {
        let current_dir = std::env::current_dir().unwrap();
        let result = Cd.execute("");
        assert!(result.is_ok());
        assert_ne!(current_dir, std::env::current_dir().unwrap())
    }

    #[test]
    fn change_to_param_dir() {
        let current_dir = std::env::current_dir().unwrap();
        let result = Cd.execute("/");
        assert!(result.is_ok());
        assert_ne!(current_dir, std::env::current_dir().unwrap())
    }

    #[test]
    fn error_not_found() {
        let result = Cd.execute("/x/y/z");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::DirectoryNotFound("/x/y/z".parse().unwrap())
        )
    }

    #[test]
    fn errorr_not_a_dir() {
        let result = Cd.execute("/bin/cat");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::NotADirectory("/bin/cat".to_string().into())
        )
    }
}
