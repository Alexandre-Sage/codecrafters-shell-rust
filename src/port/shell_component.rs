use std::sync::Arc;

use crate::{
    exceptions::{application::ApplicationError, commands::CommandError},
    port::command::CommandResult,
};

pub trait ShellComponent {
    fn execute(&self, input: &str) -> Result<CommandResult, ApplicationError> {
        let (command, args) = input.split_once(" ").unwrap_or((input, ""));

        match self.handler(command, args) {
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

    fn handler(&self, command: &str, args: &str) -> Result<CommandResult, ApplicationError>;

    fn next(&self) -> Option<Arc<dyn ShellComponent>>;
}
