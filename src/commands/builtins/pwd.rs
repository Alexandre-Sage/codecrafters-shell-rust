use crate::{
    exceptions::commands::ShellError,
    port::command::{Command, CommandResult},
};

pub struct Pwd;

impl Command for Pwd {
    fn execute(&self, _args: &[String]) -> Result<CommandResult, ShellError> {
        std::env::current_dir()
            .map(|path| CommandResult::stdout(format!("{}\n", path.to_string_lossy())))
            .map_err(|err| ShellError::Uncontroled(err.to_string()))
    }
}
