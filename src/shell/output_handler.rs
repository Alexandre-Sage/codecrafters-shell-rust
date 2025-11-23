use std::sync::Arc;

use crate::{
    exceptions::commands::CommandError,
    port::command::CommandResult,
    shell::{file::FileManager, input_parser::redirection_context::RedirectionContext},
};

pub struct OutputHandler {
    file_manager: Arc<FileManager>,
}

impl OutputHandler {
    pub fn new(file_manager: Arc<FileManager>) -> Self {
        Self { file_manager }
    }

    pub fn handle(
        &self,
        command_result: CommandResult,
        redirection: Option<RedirectionContext>,
    ) -> Result<(), CommandError> {
        match command_result {
            CommandResult::Exit(code) => std::process::exit(code),
            CommandResult::Stdio(stdout, stderr) => {
                self.write_output(&stdout, &stderr, redirection)
            }
            CommandResult::Error(error) => self.write_output("", &error.to_string(), redirection),
            CommandResult::Empty => Ok(()),
        }
    }

    fn write_stderr(&self, stderr: &str) {
        if !stderr.is_empty() {
            eprint!("{stderr}")
        }
    }

    fn write_stdout(&self, stdout: &str) {
        if !stdout.is_empty() {
            print!("{stdout}")
        }
    }

    fn write_output(
        &self,
        stdout: &str,
        stderr: &str,
        redirection: Option<RedirectionContext>,
    ) -> Result<(), CommandError> {
        if let Some(redirection) = redirection {
            if redirection.should_write_stdout() {
                self.write_stderr(stderr);
                return self.file_manager.write_to_file(&redirection.path, stdout);
            }

            if redirection.should_append_stdout() {
                self.write_stderr(stderr);
                return self.file_manager.append_to_file(&redirection.path, stdout);
            }

            if redirection.should_write_stderr() {
                self.write_stdout(stdout);
                return self.file_manager.write_to_file(&redirection.path, stderr);
            }
        }

        self.write_stderr(stderr);
        self.write_stdout(stdout);

        Ok(())
    }
}
