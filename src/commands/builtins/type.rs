use std::{path::PathBuf, str::FromStr, sync::Arc};

use crate::{
    commands::CommandToken,
    exceptions::{
        commands::CommandError::{self, TooManyArguments},
        type_command_error::TypeCommandError,
    },
    port::command::{Command, CommandResult},
    shell::path::Path,
};

pub struct Type {
    path_dirs: Arc<Path>,
}

impl Type {
    pub fn new(path_dirs: Arc<Path>) -> Self {
        Self { path_dirs }
    }
}

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

        if CommandToken::from_str(args).is_ok() {
            return Ok(CommandResult::Message(
                args.to_owned() + " is a shell builtin",
            ));
        }

        match self.path_dirs.find_executable(args) {
            Some(exe_path) => Ok(CommandResult::Message(format!(
                "{args} is {}",
                exe_path.display()
            ))),
            None => Err(TypeCommandError::NotFound(args.to_string()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{exceptions::commands::CommandError, port::command::CommandResult};

    use super::*;

    fn create_empty_path() -> Arc<Path> {
        Arc::new(Path::new(vec![]))
    }

    // Builtin command tests

    #[test]
    fn type_echo_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("echo");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("echo is a shell builtin".to_string())
        )
    }

    #[test]
    fn type_exit_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("exit");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("exit is a shell builtin".to_string())
        )
    }

    #[test]
    fn type_itself_is_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("type");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("type is a shell builtin".to_string())
        )
    }

    // Error handling tests

    #[test]
    fn type_unknown_command() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("nonexistentcommand");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TypeCommandError(TypeCommandError::NotFound(
                "nonexistentcommand".to_string()
            ))
        )
    }

    #[test]
    fn type_empty_args() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::EmptyArgs(1))
    }

    #[test]
    fn type_too_many_args() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("echo exit");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TooManyArguments("1".to_string(), 2)
        )
    }

    #[test]
    fn type_multiple_args() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute("echo exit ls");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TooManyArguments("1".to_string(), 3)
        )
    }

    // PATH searching tests

    #[test]
    fn type_finds_ls_in_system_path() {
        // Use actual system PATH
        let paths = Arc::new(Path::from_env());
        let result = Type::new(paths).execute("ls");

        // ls should be found in PATH (exists on most Unix systems)
        assert!(result.is_ok());
        let msg = result.unwrap();
        if let CommandResult::Message(s) = msg {
            assert!(s.starts_with("ls is "));
            assert!(s.contains("/ls"));
        } else {
            panic!("Expected Message variant");
        }
    }

    #[test]
    fn type_finds_cat_in_system_path() {
        let paths = Arc::new(Path::from_env());
        let result = Type::new(paths).execute("cat");

        assert!(result.is_ok());
        let msg = result.unwrap();
        if let CommandResult::Message(s) = msg {
            assert!(s.starts_with("cat is "));
            assert!(s.contains("/cat"));
        } else {
            panic!("Expected Message variant");
        }
    }

    #[test]
    fn type_nonexistent_external_command() {
        let paths = Arc::new(Path::from_env());
        let result = Type::new(paths).execute("thisdoesnotexist12345");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommandError::TypeCommandError(TypeCommandError::NotFound(
                "thisdoesnotexist12345".to_string()
            ))
        )
    }

    #[test]
    fn type_with_specific_paths() {
        let paths = Arc::new(Path::new(vec![
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ]));

        let result = Type::new(paths).execute("ls");

        assert!(result.is_ok());
    }

    #[test]
    fn type_builtin_takes_precedence_over_external() {
        let paths = Arc::new(Path::from_env());
        let result = Type::new(paths).execute("echo");

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("echo is a shell builtin".to_string())
        );
    }

    #[test]
    fn type_empty_path_only_finds_builtins() {
        let paths = create_empty_path();

        let result = Type::new(Arc::clone(&paths)).execute("echo");
        assert!(result.is_ok());

        let result = Type::new(paths).execute("ls");
        assert!(result.is_err());
    }
}
