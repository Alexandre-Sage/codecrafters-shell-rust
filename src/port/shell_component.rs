use std::sync::Arc;

use crate::{
    exceptions::commands::ShellError, port::command::CommandResult,
    shell::input::input_parser::ParsedCommand,
};

pub trait ShellComponent {
    fn execute(&self, input: ParsedCommand) -> Result<CommandResult, ShellError> {
        match self.handler(input.command(), input.args()) {
            Ok(res) => Ok(res),
            Err(err) => {
                if let Some(next) = self.next() {
                    if matches!(err, ShellError::CommandNotFound(_)) {
                        return next.execute(input);
                    }
                }
                Err(err)
            }
        }
    }

    fn handler(&self, command: &str, args: &[String]) -> Result<CommandResult, ShellError>;

    fn next(&self) -> Option<Arc<dyn ShellComponent>>;
}
