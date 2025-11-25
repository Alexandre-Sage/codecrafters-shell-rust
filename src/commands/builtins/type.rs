use std::{str::FromStr, sync::Arc};

use crate::{
    commands::CommandToken,
    exceptions::{
        commands::ShellError::{self, TooManyArguments},
        type_command_error::TypeCommandError,
    },
    port::command::{Command, CommandResult},
    shell::path::PathDirs,
};

pub struct Type {
    path_dirs: Arc<PathDirs>,
}

impl Type {
    pub fn new(path_dirs: Arc<PathDirs>) -> Self {
        Self { path_dirs }
    }
}

impl Command for Type {
    fn execute(
        &self,
        args: &[String],
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::ShellError> {
        if args.is_empty() {
            return Err(ShellError::EmptyArgs(1));
        }

        if args.len() > 1 {
            return Err(TooManyArguments("1".to_string(), args.len()));
        }
        let arg = &args[0];

        if CommandToken::from_str(arg).is_ok() {
            return Ok(CommandResult::stdout(format!(
                "{} is a shell builtin\n",
                arg
            )));
        }

        match self.path_dirs.find_executable(arg) {
            Some(exe_path) => Ok(CommandResult::stdout(format!(
                "{arg} is {}\n",
                exe_path.display(),
            ))),
            None => Err(TypeCommandError::NotFound(arg.to_string()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{exceptions::commands::ShellError, port::command::CommandResult};
    use std::path::PathBuf;

    use super::*;

    fn create_empty_path() -> Arc<PathDirs> {
        Arc::new(PathDirs::new(vec![]))
    }

    // Builtin command tests

    #[test]
    fn type_echo_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&["echo".to_string()]);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::stdout("echo is a shell builtin\n")
        )
    }

    #[test]
    fn type_exit_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&["exit".to_string()]);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::stdout("exit is a shell builtin\n")
        )
    }

    #[test]
    fn type_itself_is_builtin() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&["type".to_string()]);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::stdout("type is a shell builtin\n")
        )
    }

    // Error handling tests

    #[test]
    fn type_unknown_command() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&["nonexistentcommand".to_string()]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShellError::TypeCommandError(TypeCommandError::NotFound(
                "nonexistentcommand".to_string()
            ))
        )
    }

    #[test]
    fn type_empty_args() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ShellError::EmptyArgs(1))
    }

    #[test]
    fn type_too_many_args() {
        let paths = create_empty_path();
        let result = Type::new(paths).execute(&["echo".to_string(), "exit".to_string()]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShellError::TooManyArguments("1".to_string(), 2)
        )
    }

    #[test]
    fn type_multiple_args() {
        let paths = create_empty_path();
        let result =
            Type::new(paths).execute(&["echo".to_string(), "exit".to_string(), "ls".to_string()]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShellError::TooManyArguments("1".to_string(), 3)
        )
    }

    // PATH searching tests

    #[test]
    fn type_finds_ls_in_system_path() {
        // Use actual system PATH
        let paths = Arc::new(PathDirs::from_env());
        let result = Type::new(paths).execute(&["ls".to_string()]);

        // ls should be found in PATH (exists on most Unix systems)
        assert!(result.is_ok());
        let msg = result.unwrap();
        if let CommandResult::Stdio(s, _) = msg {
            assert!(s.starts_with("ls is "));
            assert!(s.contains("/ls"));
        } else {
            panic!("Expected Message variant");
        }
    }

    #[test]
    fn type_finds_cat_in_system_path() {
        let paths = Arc::new(PathDirs::from_env());
        let result = Type::new(paths).execute(&["cat".to_string()]);

        assert!(result.is_ok());
        let msg = result.unwrap();
        if let CommandResult::Stdio(s, _) = msg {
            assert!(s.starts_with("cat is "));
            assert!(s.contains("/cat"));
        } else {
            panic!("Expected Message variant");
        }
    }

    #[test]
    fn type_nonexistent_external_command() {
        let paths = Arc::new(PathDirs::from_env());
        let result = Type::new(paths).execute(&["thisdoesnotexist12345".to_string()]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ShellError::TypeCommandError(TypeCommandError::NotFound(
                "thisdoesnotexist12345".to_string()
            ))
        )
    }

    #[test]
    fn type_with_specific_paths() {
        let paths = Arc::new(PathDirs::new(vec![
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ]));

        let result = Type::new(paths).execute(&["ls".to_string()]);

        assert!(result.is_ok());
    }

    #[test]
    fn type_builtin_takes_precedence_over_external() {
        let paths = Arc::new(PathDirs::from_env());
        let result = Type::new(paths).execute(&["echo".to_string()]);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::stdout("echo is a shell builtin\n")
        );
    }

    #[test]
    fn type_empty_path_only_finds_builtins() {
        let paths = create_empty_path();

        let result = Type::new(Arc::clone(&paths)).execute(&["echo".to_string()]);
        assert!(result.is_ok());

        let result = Type::new(paths).execute(&["ls".to_string()]);
        assert!(result.is_err());
    }
}
