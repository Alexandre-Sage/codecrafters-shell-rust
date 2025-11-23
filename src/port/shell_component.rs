use std::sync::Arc;

use crate::{
    exceptions::{application::ApplicationError, commands::CommandError},
    port::command::CommandResult,
    shell::input_parser::ParsedCommand,
};

pub trait ShellComponent {
    fn execute(&self, input: ParsedCommand) -> Result<CommandResult, CommandError> {
        match self.handler(input.command(), input.args()) {
            Ok(res) => Ok(res),
            Err(err) => {
                if let Some(next) = self.next() {
                    if matches!(err, CommandError::CommandNotFound(_)) {
                        return next.execute(input);
                    }
                }
                Err(err)
            }
        }
    }

    fn handler(&self, command: &str, args: &[String]) -> Result<CommandResult, CommandError>;

    fn next(&self) -> Option<Arc<dyn ShellComponent>>;
}
