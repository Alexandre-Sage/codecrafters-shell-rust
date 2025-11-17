use std::sync::Arc;

use crate::{
    exceptions::{application::ApplicationError, commands::CommandError},
    parser::ParsedCommand,
    port::command::CommandResult,
};

pub trait ShellComponent {
    fn execute(&self, input: ParsedCommand) -> Result<CommandResult, ApplicationError> {
        match self.handler(input.command(), input.args()) {
            Ok(res) => Ok(res),
            Err(err) => {
                if let Some(next) = self.next() {
                    if matches!(
                        err,
                        ApplicationError::CommandError(CommandError::CommandNotFound(_))
                    ) {
                        return next.execute(input);
                    }
                }
                Err(err)
            }
        }
    }

    fn handler(&self, command: &str, args: &[String]) -> Result<CommandResult, ApplicationError>;

    fn next(&self) -> Option<Arc<dyn ShellComponent>>;
}
