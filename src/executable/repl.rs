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
        input_parser::{
            redirection_context::{self, RedirectionChannel, RedirectionContext, RedirectionType},
            InputParser,
        },
        path::Path,
        redirection,
    },
};

pub struct Repl {
    builtins: CommandRegistry,
    input_parser: InputParser,
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
            file_manager,
        }
    }

    fn prompt(&self) -> Result<(), io::Error> {
        print!("$ ");

        io::stdout().flush()
    }

    fn handle_output(
        &self,
        command_result: CommandResult,
        redirection: Option<RedirectionContext>,
    ) -> Result<(), CommandError> {
        match command_result {
            CommandResult::Exit(code) => {
                std::process::exit(code);
            }
            CommandResult::Message(message) => {
                if let Some(redirection_context) = redirection {
                    return self
                        .file_manager
                        .write_to_file(&redirection_context.path, message);
                }
                print!("{message}")
            }
            CommandResult::Stdio(stdout, stderr) => {
                if let Some(redirection_context) = redirection {
                    if matches!(
                        redirection_context.redirection_type,
                        RedirectionType::Output(RedirectionChannel::Stdout)
                    ) {
                        eprint!("{stderr}");
                        return self
                            .file_manager
                            .write_to_file(&redirection_context.path, stdout);
                    }
                }
                print!("{stdout}");
                eprint!("{stderr}");
            }
            CommandResult::Empty => {
                // Command executed successfully with no output (like cd)
            }
        };
        Ok(())
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
                    eprintln!("{err}");
                }
                Ok(res) => self.handle_output(res, redirection)?,
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
