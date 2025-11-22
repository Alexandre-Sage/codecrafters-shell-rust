use std::{path::PathBuf, sync::Arc};

use crate::{
    exceptions::commands::CommandError,
    port::command::{Command, CommandResult},
    shell::file::FileManager,
};

pub struct Cd {
    file_manager: Arc<FileManager>,
}

impl Cd {
    pub fn new(file_manager: Arc<FileManager>) -> Self {
        Self { file_manager }
    }

    fn change_dir(
        &self,
        dir: PathBuf,
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        std::env::set_current_dir(dir).map_err(|err| CommandError::Unknown(err.to_string()))?;

        Ok(CommandResult::Empty)
    }
}

impl Command for Cd {
    fn execute(
        &self,
        args: &[String],
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        if args.len() > 1 {
            return Err(CommandError::TooManyArguments("1".to_string(), args.len()));
        }

        let path = args.get(0).map_or("", |s| s.as_str());
        let path = self.file_manager.handle_path(path)?;

        if !path.is_dir() {
            return Err(CommandError::NotADirectory(path));
        }

        self.change_dir(path)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn cd_command() -> Cd {
        let file_manager = Arc::new(FileManager);
        Cd::new(file_manager)
    }

    #[test]
    fn change_to_home_if_empty() {
        let current_dir = std::env::current_dir().unwrap();
        let result = cd_command().execute(&[]);
        assert!(result.is_ok());
        assert_ne!(current_dir, std::env::current_dir().unwrap())
    }

    #[test]
    fn change_to_param_dir() {
        let current_dir = std::env::current_dir().unwrap();
        let result = cd_command().execute(&["/".to_string()]);
        assert!(result.is_ok());
        assert_ne!(current_dir, std::env::current_dir().unwrap())
    }

    #[test]
    fn error_not_found() {
        let result = cd_command().execute(&["/x/y/z".to_string()]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::DirectoryNotFound("/x/y/z".parse().unwrap())
        )
    }

    #[test]
    fn errorr_not_a_dir() {
        let result = cd_command().execute(&["/bin/cat".to_string()]);
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
        let result = cd_command().execute(&["~".to_string()]);
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
