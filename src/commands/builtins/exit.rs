use crate::{
    exceptions::commands::ShellError,
    port::command::{Command, CommandResult},
};

pub struct Exit;

impl Command for Exit {
    fn execute(
        &self,
        args: &[String],
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::ShellError> {
        if args.len() > 1 {
            return Err(ShellError::TooManyArguments(
                "at most 1".to_string(),
                args.len(),
            ));
        }

        let arg = if !args.is_empty() {
            args[0]
                .parse()
                .map_err(|_| ShellError::ParsingError("integer".to_string()))?
        } else {
            0
        };

        Ok(CommandResult::Exit(arg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_0() {
        let command = Exit;
        let result = command.execute(&["0".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::Exit(0))
    }
    #[test]
    fn exit_1() {
        let command = Exit;
        let result = command.execute(&["1".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::Exit(1))
    }
}
