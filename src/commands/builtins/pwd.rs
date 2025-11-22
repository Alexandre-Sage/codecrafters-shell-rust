use crate::{
    exceptions::commands::CommandError,
    port::command::{Command, CommandResult},
};

pub struct Pwd;

impl Command for Pwd {
    fn execute(&self, _args: &[String]) -> Result<CommandResult, CommandError> {
        std::env::current_dir()
            .map(|path| CommandResult::Message(path.to_string_lossy().to_string() + "\n"))
            .map_err(|err| CommandError::Unknown(err.to_string()))
    }
}
