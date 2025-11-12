use std::{collections::HashMap, sync::Arc};

use crate::{
    commands::CommandToken,
    exceptions::commands::CommandError,
    port::command::Command,
};

#[derive(Default)]
pub(crate) struct CommandRegistry(HashMap<CommandToken, Arc<dyn Command>>);

impl CommandRegistry {
    pub fn try_get(&self, command: &str) -> Result<&Arc<dyn Command>, CommandError> {
        let token = command.parse()?;

        self.0
            .get(&token)
            .ok_or(CommandError::CommandNotFound(command.to_owned()))
    }

    pub fn register(&mut self, token: CommandToken, command: Arc<dyn Command>) {
        self.0.insert(token, command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCommand;
    impl Command for FakeCommand {
        fn execute(&self, _args: &str) -> Result<command::CommandResult, CommandError> {
            todo!()
        }
    }

    #[test]
    fn get_command() {
        let mut registry = CommandRegistry::default();
        registry.register(CommandToken::Exit, Arc::new(FakeCommand));
        let result = registry.try_get("exit");
        assert!(result.is_ok())
    }

    #[test]
    fn command_not_found() {
        let registry = CommandRegistry::default();
        let result = registry.try_get("exit");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CommandError::CommandNotFound("exit".to_owned())
        )
    }
}
