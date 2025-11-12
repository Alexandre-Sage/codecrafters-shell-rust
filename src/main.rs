pub(crate) mod commands;
pub(crate) mod exceptions;
pub(crate) mod executable;
pub(crate) mod port;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::sync::Arc;

use crate::{
    commands::{
        builtins::exit::ExitCommand,
        registry::{self, CommandRegistry},
        CommandToken,
    },
    exceptions::{application::ApplicationError, commands::CommandError},
    port::command::CommandResult,
};

struct Repl {
    builtins: CommandRegistry,
}

impl Repl {
    fn new() -> Self {
        let mut registry = CommandRegistry::default();
        registry.register(CommandToken::Exit, Arc::new(ExitCommand));

        Self { builtins: registry }
    }

    fn prompt(&self) -> Result<(), io::Error> {
        print!("$ ");

        io::stdout().flush()
    }

    fn parse_arg(args: &str) -> (&str, Vec<&str>) {
        let parts = args.split_whitespace().collect::<Vec<&str>>();
        let command = parts[0].trim();

        if parts.len() > 1 {
            return (
                command,
                parts[1..parts.len()].iter().map(|arg| arg.trim()).collect(),
            );
        }

        (command, vec![])
    }

    fn spawn(&self) -> Result<(), CommandError> {
        loop {
            self.prompt().unwrap();

            let mut buffer = String::new();

            io::stdin().read_line(&mut buffer).unwrap();

            let (command, args) = Self::parse_arg(&buffer);

            let command = self
                .builtins
                .try_get(&command)
                .and_then(|command| command.execute(&args));

            match command {
                Err(err) => {
                    eprintln!("{}", err);
                }
                Ok(res) => match res {
                    CommandResult::Exit(code) => {
                        std::process::exit(code);
                    }
                },
            };

            // io::stdout().write(b"\n").unwrap();
            io::stdout().flush().unwrap();
        }
    }
}

fn main() {
    Repl::new().spawn().unwrap();
}
