use std::char;
use std::io::{self, stdout, Read, Stdin, Stdout, StdoutLock, Write};
use std::sync::Arc;

use anyhow::Result;

use crate::shell::completion;
use crate::shell::completion::builtins::BuiltinsCompletion;
use crate::shell::input::input_handler::{InputHandler, InputResult};
use crate::shell::raw_mode::RawMode;
use crate::{
    commands::{
        builtins::{cd::Cd, echo::Echo, exit::Exit, pwd::Pwd, r#type::Type},
        registry::CommandRegistry,
        CommandToken,
    },
    exceptions::commands::ShellError,
    external::ExternalCommand,
    port::{command::CommandResult, shell_component::ShellComponent},
    shell::{
        file::FileManager, input::input_parser::InputParser, output_handler::OutputHandler,
        path::PathDirsProvider,
    },
};

pub struct Repl {
    builtins: CommandRegistry,
    input_parser: InputParser,
    output_handler: Arc<OutputHandler>,
    input_handler: InputHandler,
    // file_manager: Arc<FileManager>,
}

impl Repl {
    pub fn new(file_manager: Arc<FileManager>, output_handler: Arc<OutputHandler>) -> Self {
        let path_dirs = Arc::new(PathDirsProvider::from_env());
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

        let completions = BuiltinsCompletion::new(Arc::clone(&path_dirs));

        Self {
            builtins: registry,
            input_parser: InputParser::new(Arc::clone(&file_manager)),
            input_handler: InputHandler::new(completions), // file_manager,
            output_handler,
        }
    }

    fn prompt(&self, content: Option<String>) -> Result<(), ShellError> {
        let prompt = match content {
            Some(content) => &format!("$ {content}"),
            None => "$ ",
        };
        print!("{prompt}");

        io::stdout()
            .flush()
            .map_err(|err| ShellError::Uncontroled(err.to_string()))
    }

    pub fn spawn(&self) -> Result<(), ShellError> {
        let mut previous_content: Option<String> = None;
        loop {
            self.prompt(previous_content.clone())?;

            let input = self.input_handler.handle(previous_content.clone())?;

            if previous_content.is_some() {
                previous_content = None
            }

            match input {
                InputResult::Reset => self.output_handler.write_stdout("^C\r\n"),
                InputResult::MultiCompletion {
                    completion_items,
                    input,
                } => {
                    self.output_handler.write_stdout(&completion_items);
                    previous_content = Some(input);
                    // self.prompt(Some(&input))?;
                }
                InputResult::Input(buffer) => {
                    let (parsed_command, redirection) = self.input_parser.parse(buffer.trim())?;
                    let command = self.builtins.execute(parsed_command);
                    match command {
                        Err(err) => self
                            .output_handler
                            .handle(CommandResult::Error(err), redirection),
                        Ok(res) => self.output_handler.handle(res, redirection),
                    }?;
                }
            }

            io::stderr()
                .flush()
                .map_err(|err| ShellError::Uncontroled(err.to_string()))?;
            io::stdout()
                .flush()
                .map_err(|err| ShellError::Uncontroled(err.to_string()))?;
        }
    }
}
