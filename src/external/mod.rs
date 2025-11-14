use std::{borrow::Cow, process::Stdio, sync::Arc};

use crate::{
    exceptions::{application::ApplicationError, commands::CommandError},
    port::{command::CommandResult, shell_component::ShellComponent},
    shell::path::Path,
};

pub struct ExternalCommand {
    path_dirs: Arc<Path>,
}

impl ExternalCommand {
    pub fn new(path_dirs: Arc<Path>) -> Self {
        Self { path_dirs }
    }
}

impl ShellComponent for ExternalCommand {
    fn handler(&self, command: &str, args: &str) -> Result<CommandResult, ApplicationError> {
        if let Some(exe_path) = self.path_dirs.find_executable(command) {
            let output = std::process::Command::new(exe_path)
                .args(args.split_whitespace())
                .output()
                // .stdout(Stdio::inherit())
                // .stderr(Stdio::inherit())
                // .status()
                .map_err(|err| CommandError::ExternalError(err.to_string()))?;

            return Ok(CommandResult::Message(
                String::from_utf8_lossy(&output.stdout).to_string(),
            ));
        }

        Err(CommandError::CommandNotFound(command.to_owned()).into())
    }

    fn next(&self) -> Option<Arc<dyn ShellComponent>> {
        None
    }
}
