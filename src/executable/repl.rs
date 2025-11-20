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
    parser::InputParser,
    port::{command::CommandResult, shell_component::ShellComponent},
    shell::path::Path,
};

pub struct Repl {
    builtins: CommandRegistry,
    input_parser: InputParser,
}

impl Repl {
    pub fn new() -> Self {
        let paths = Arc::new(Path::from_env());
        let mut registry = CommandRegistry::new(paths.clone());
        registry.register(CommandToken::Exit, Arc::new(Exit));
        registry.register(CommandToken::Echo, Arc::new(Echo));
        registry.register(CommandToken::Type, Arc::new(Type::new(paths.clone())));
        registry.register(CommandToken::Pwd, Arc::new(Pwd));
        registry.register(CommandToken::Cd, Arc::new(Cd));

        Self {
            builtins: registry,
            input_parser: InputParser::new(),
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

            let parsed_command = match self.input_parser.parse(buffer.trim()) {
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
                Ok(res) => match res {
                    CommandResult::Exit(code) => {
                        std::process::exit(code);
                    }
                    CommandResult::Message(message) => {
                        println!("{message}")
                    }
                    CommandResult::Stdio(stdout, stderr) => {
                        print!("{stdout}");
                        eprint!("{stderr}");
                    }
                    CommandResult::Empty => {
                        // Command executed successfully with no output (like cd)
                        ()
                    }
                },
            };

            io::stdout().flush().unwrap();
        }
    }
}
