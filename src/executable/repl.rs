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
        output_handler::{self, OutputHandler},
        path::Path,
    },
};

pub struct Repl {
    builtins: CommandRegistry,
    input_parser: InputParser,
    output_handler: Arc<OutputHandler>,
    file_manager: Arc<FileManager>,
}

impl Repl {
    pub fn new(file_manager: Arc<FileManager>, output_handler: Arc<OutputHandler>) -> Self {
        let path_dirs = Arc::new(Path::from_env());
        let external_command = Arc::new(ExternalCommand::new(Arc::clone(&path_dirs)));
        let mut registry = CommandRegistry::new(Arc::clone(&path_dirs), external_command);
        registry.register(CommandToken::Exit, Arc::new(Exit));
        registry.register(CommandToken::Echo, Arc::new(Echo));
        registry.register(
            CommandToken::Type,
            Arc::new(Type::new(Arc::clone(&path_dirs))),
        );
        registry.register(CommandToken::Pwd, Arc::new(Pwd));
        registry.register(
            CommandToken::Cd,
            Arc::new(Cd::new(Arc::clone(&file_manager))),
        );

        Self {
            builtins: registry,
            input_parser: InputParser::new(Arc::clone(&file_manager)),
            output_handler,
            file_manager,
        }
    }

    fn prompt(&self) -> Result<(), io::Error> {
        print!("$ ");

        io::stdout().flush()
    }

    pub fn spawn(&self) -> Result<(), CommandError> {
        loop {
            self.prompt()
                .map_err(|err| CommandError::Unknown(err.to_string()))?;

            let mut buffer = String::new();

            io::stdin()
                .read_line(&mut buffer)
                .map_err(|err| CommandError::Unknown(err.to_string()))?;

            let (parsed_command, redirection) = match self.input_parser.parse(buffer.trim()) {
                Ok(cmd) => cmd,
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
            };

            let command = self.builtins.execute(parsed_command);
            match command {
                Err(err) => self
                    .output_handler
                    .handle(CommandResult::Error(err), redirection),
                Ok(res) => self.output_handler.handle(res, redirection),
            }?;

            io::stderr()
                .flush()
                .map_err(|err| CommandError::Unknown(err.to_string()))?;
            io::stdout()
                .flush()
                .map_err(|err| CommandError::Unknown(err.to_string()))?;
        }
    }
}
