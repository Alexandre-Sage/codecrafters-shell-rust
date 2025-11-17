use std::path::PathBuf;

use crate::{
    exceptions::commands::CommandError,
    port::command::{Command, CommandResult},
};

pub struct Cd;

impl Cd {
    fn change_dir(
        &self,
        dir: PathBuf,
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        std::env::set_current_dir(dir).map_err(|err| CommandError::Unknown(err.to_string()))?;

        Ok(CommandResult::Empty)
    }

    fn should_go_to_homedir(&self, args: &[String]) -> bool {
        if args.is_empty() {
            return true;
        }
        let arg = &args[0];
        arg == "~" || arg == "~/"
    }

    fn format_path(&self, args: &str) -> Result<PathBuf, CommandError> {
        let home_dir = std::env::home_dir().unwrap();

        if let Some(remaining) = args.strip_prefix("~/") {
            return Ok(home_dir.join(remaining));
        }

        // if args.starts_with("~") {
        //     let remaining = &args[1..];
        //     return Ok(home_dir.join(remaining));
        // }

        Ok(PathBuf::from(args.trim()))
    }
}

impl Command for Cd {
    fn execute(
        &self,
        args: &[String],
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        let home_dir = std::env::home_dir().unwrap();

        if self.should_go_to_homedir(args) {
            return self.change_dir(home_dir);
        }

        // let args_parts: Vec<_> = args.split_whitespace().collect();

        if args.len() > 1 {
            return Err(CommandError::TooManyArguments("1".to_string(), args.len()));
        }

        let path = self.format_path(&args[0])?;

        if !path.exists() {
            return Err(CommandError::DirectoryNotFound(path));
        }

        if !path.is_dir() {
            return Err(CommandError::NotADirectory(path));
        }

        self.change_dir(path)
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

    #[test]
    fn change_to_home_with_tilde() {
        // Save current directory to restore later
        let original_dir = std::env::current_dir().unwrap();

        // Execute cd ~
        let result = Cd.execute("~");
        assert!(result.is_ok(), "cd ~ should succeed");

        // Verify we're actually in home directory
        let current_dir = std::env::current_dir().unwrap();
        let home_dir = std::env::home_dir().unwrap();
        assert_eq!(
            current_dir, home_dir,
            "Current directory should be home directory after cd ~"
        );

        // Restore original directory for other tests
        std::env::set_current_dir(original_dir).unwrap();
    }
}
