use crate::port::command::{Command, CommandResult};

pub struct Echo;

impl Command for Echo {
    fn execute(
        &self,
        args: &[String],
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::ShellError> {
        Ok(CommandResult::stdout(format!("{}\n", args.join(" "))))
    }
}
#[cfg(test)]
mod tests {
    use crate::port::command::CommandResult;

    use super::*;

    #[test]
    fn echo_hello_world() {
        let result = Echo.execute(&["hello".to_string(), "world".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::stdout("hello world\n"))
    }

    #[test]
    fn echo_empty() {
        let result = Echo.execute(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::stdout("\n"))
    }
}
