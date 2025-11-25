use std::{collections::HashMap, sync::Arc};

use crate::{
    commands::CommandToken,
    exceptions::commands::ShellError,
    port::{
        command::{Command, CommandResult},
        shell_component::ShellComponent,
    },
    shell::path::Path,
};

pub(crate) struct CommandRegistry {
    registry: HashMap<CommandToken, Arc<dyn Command>>,
    path_dirs: Arc<Path>,
    next: Arc<dyn ShellComponent>,
}

impl CommandRegistry {
    pub(crate) fn new(path_dirs: Arc<Path>, next: Arc<dyn ShellComponent>) -> Self {
        Self {
            registry: HashMap::default(),
            path_dirs,
            next,
        }
    }

    pub fn try_get(&self, command: &str) -> Result<&Arc<dyn Command>, ShellError> {
        let token = command.parse()?;

        self.registry
            .get(&token)
            .ok_or(ShellError::CommandNotFound(command.to_owned()))
    }

    pub fn register(&mut self, token: CommandToken, command: Arc<dyn Command>) {
        self.registry.insert(token, command);
    }
}

impl ShellComponent for CommandRegistry {
    fn handler(&self, command: &str, args: &[String]) -> Result<CommandResult, ShellError> {
        let command = self.try_get(command)?;
        let result = command.execute(args)?;

        Ok(result)
    }
    fn next(&self) -> Option<Arc<dyn ShellComponent>> {
        Some(Arc::clone(&self.next))
    }
}
#[cfg(test)]
mod tests {
    use crate::{external::ExternalCommand, port::command::CommandResult};

    use super::*;

    struct FakeCommand;
    impl Command for FakeCommand {
        fn execute(&self, _args: &[String]) -> Result<CommandResult, ShellError> {
            todo!()
        }
    }

    #[test]
    fn get_command() {
        let paths = Arc::new(Path::new(vec![]));
        let external = Arc::new(ExternalCommand::new(paths.clone()));
        let mut registry = CommandRegistry::new(paths, external);
        registry.register(CommandToken::Exit, Arc::new(FakeCommand));
        let result = registry.try_get("exit");
        assert!(result.is_ok())
    }

    #[test]
    fn command_not_found() {
        let paths = Arc::new(Path::new(vec![]));
        let external = Arc::new(ExternalCommand::new(paths.clone()));
        let registry = CommandRegistry::new(paths, external);
        let result = registry.try_get("exit");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ShellError::CommandNotFound("exit".to_owned())
        )
    }
}
