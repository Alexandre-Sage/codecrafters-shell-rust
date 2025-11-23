#[allow(unused_imports)]
use std::io::{self, Write};
use std::sync::Arc;

use anyhow::Result;

use crate::{
    commands::{
        builtins::{cd::Cd, echo::Echo, exit::Exit, pwd::Pwd, r#type::Type},
        registry::CommandRegistry,
        CommandToken,
    },
    exceptions::commands::CommandError,
    external::ExternalCommand,
    port::{command::CommandResult, shell_component::ShellComponent},
    shell::{
        file::FileManager,
        input_parser::{redirection_context::RedirectionContext, InputParser},
        path::Path,
    },
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

pub struct Repl {
    builtins: CommandRegistry,
    input_parser: InputParser,
    output_handler: OutputHandler,
    file_manager: Arc<FileManager>,
}

impl Repl {
    pub fn new() -> Self {
        let path_dirs = Arc::new(Path::from_env());
        let file_manager = FileManager.into();
        let external_command = Arc::new(ExternalCommand::new(Arc::clone(&path_dirs)));
        let mut registry = CommandRegistry::new(path_dirs.clone(), external_command);
        registry.register(CommandToken::Exit, Arc::new(Exit));
        registry.register(CommandToken::Echo, Arc::new(Echo));
        registry.register(CommandToken::Type, Arc::new(Type::new(path_dirs.clone())));
        registry.register(CommandToken::Pwd, Arc::new(Pwd));
        registry.register(
            CommandToken::Cd,
            Arc::new(Cd::new(Arc::clone(&file_manager))),
        );

        Self {
            builtins: registry,
            input_parser: InputParser::new(Arc::clone(&file_manager)),
            output_handler: OutputHandler::new(Arc::clone(&file_manager)),
            file_manager,
        }
    }

    fn prompt(&self) -> Result<(), io::Error> {
        print!("$ ");

        io::stdout().flush()
    }

    pub fn spawn(&self) -> Result<(), CommandError> {
        loop {
            self.prompt().unwrap();

            let mut buffer = String::new();

            io::stdin().read_line(&mut buffer).unwrap();

            let (parsed_command, redirection) = match self.input_parser.parse(buffer.trim()) {
                Ok(cmd) => cmd,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            let command = self.builtins.execute(parsed_command);
            match command {
                Err(err) => {
                    if let Some(redirection_context) = redirection {
                        if redirection_context.should_write_stderr() {
                            return self
                                .file_manager
                                .write_to_file(&redirection_context.path, err.to_string());
                        }
                    } else {
                        eprintln!("{err}");
                    }
                }
                Ok(res) => self.output_handler.handle(res, redirection)?,
            };

            io::stderr()
                .flush()
                .map_err(|err| CommandError::Unknown(err.to_string()))?;
            io::stdout()
                .flush()
                .map_err(|err| CommandError::Unknown(err.to_string()))?;
        }
    }
}
