use std::{collections::HashMap, sync::Arc};

use crate::{
    commands::CommandToken,
    exceptions::{application::ApplicationError, commands::CommandError},
    external::ExternalCommand,
    port::{
        command::{self, Command, CommandResult},
        shell_component::ShellComponent,
    },
    shell::path::Path,
};

pub(crate) struct CommandRegistry {
    registry: HashMap<CommandToken, Arc<dyn Command>>,
    path_dirs: Arc<Path>,
}

impl CommandRegistry {
    pub(crate) fn new(path_dirs: Arc<Path>) -> Self {
        Self {
            registry: HashMap::default(),
            path_dirs,
        }
    }

    pub fn try_get(&self, command: &str) -> Result<&Arc<dyn Command>, CommandError> {
        let token = command.parse()?;

        self.registry
            .get(&token)
            .ok_or(CommandError::CommandNotFound(command.to_owned()))
    }

    pub fn register(&mut self, token: CommandToken, command: Arc<dyn Command>) {
        self.registry.insert(token, command);
    }
}

impl ShellComponent for CommandRegistry {
    fn handler(&self, command: &str, args: &str) -> Result<CommandResult, ApplicationError> {
        let command = self.try_get(command)?;
        let result = command.execute(args)?;

        Ok(result)
    }
    fn next(&self) -> Option<Arc<dyn ShellComponent>> {
        Some(Arc::new(ExternalCommand::new(Arc::clone(&self.path_dirs))))
    }
}
#[cfg(test)]
mod tests {
    use crate::port::command::CommandResult;

    use super::*;

    struct FakeCommand;
    impl Command for FakeCommand {
        fn execute(&self, _args: &str) -> Result<CommandResult, CommandError> {
            todo!()
        }
    }

    #[test]
    fn get_command() {
        let paths = Arc::new(Path::new(vec![]));
        let mut registry = CommandRegistry::new(paths);
        registry.register(CommandToken::Exit, Arc::new(FakeCommand));
        let result = registry.try_get("exit");
        assert!(result.is_ok())
    }

    #[test]
    fn command_not_found() {
        let paths = Arc::new(Path::new(vec![]));
        let registry = CommandRegistry::new(paths);
        let result = registry.try_get("exit");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CommandError::CommandNotFound("exit".to_owned())
        )
    }
}
