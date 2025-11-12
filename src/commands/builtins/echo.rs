use crate::port::command::{Command, CommandResult};

pub struct Echo;

impl Command for Echo {
    fn execute(
        &self,
        args: &str,
    ) -> Result<crate::port::command::CommandResult, crate::exceptions::commands::CommandError>
    {
        Ok(CommandResult::Message(args.to_string()))
    }
}
#[cfg(test)]
mod tests {
    use crate::port::command::CommandResult;

    use super::*;

    #[test]
    fn echo_hello_world() {
        let result = Echo.execute("hello world");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            CommandResult::Message("hello world".to_string())
        )
    }

    fn echo_empty() {
        let result = Echo.execute("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::Message("".to_string()))
    }
}
