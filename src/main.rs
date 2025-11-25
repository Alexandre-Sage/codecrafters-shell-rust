use std::sync::Arc;

use crate::{
    executable::repl::Repl,
    port::command::CommandResult,
    shell::{file::FileManager, output_handler::OutputHandler},
};

pub(crate) mod commands;
pub(crate) mod exceptions;
pub(crate) mod executable;
pub mod external;
pub(crate) mod port;
pub mod shell;

fn main() -> Result<(), exceptions::commands::ShellError> {
    let file_manager = FileManager.into();
    let output_handler = OutputHandler::new(Arc::clone(&file_manager)).into();

    if let Err(err) = Repl::new(file_manager, Arc::clone(&output_handler)).spawn() {
        let error = CommandResult::Error(err);
        return output_handler.handle(error, None);
    }
    Ok(())
}
