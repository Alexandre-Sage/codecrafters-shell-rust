use std::sync::Arc;

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
    fn handler(&self, command: &str, args: &[String]) -> Result<CommandResult, CommandError> {
        if self.path_dirs.find_executable(command).is_some() {
            let output = std::process::Command::new(command)
                .args(args)
                // .stdout(Stdio::inherit())
                // .stderr(Stdio::inherit())
                // .status()
                .output()
                .map_err(|err| CommandError::ExternalError(err.to_string()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Ok(CommandResult::Stdio(stdout.to_string(), stderr.to_string()));
        }

        Err(CommandError::CommandNotFound(command.to_owned()).into())
    }

    fn next(&self) -> Option<Arc<dyn ShellComponent>> {
        None
    }
}
