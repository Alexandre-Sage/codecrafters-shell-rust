#[allow(unused_imports)]
use std::io::{self, Write};
use std::sync::Arc;

use anyhow::Result;

use crate::{
    commands::{
        builtins::{echo::Echo, exit::Exit, pwd::Pwd, r#type::Type},
        registry::CommandRegistry,
        CommandToken,
    },
    exceptions::commands::CommandError,
    port::{command::CommandResult, shell_component::ShellComponent},
    shell::path::Path,
};

pub struct Repl {
    builtins: CommandRegistry,
    paths: Arc<Path>,
}

impl Repl {
    pub fn new() -> Self {
        let paths = Arc::new(Path::from_env());
        let mut registry = CommandRegistry::new(paths.clone());
        registry.register(CommandToken::Exit, Arc::new(Exit));
        registry.register(CommandToken::Echo, Arc::new(Echo));
        registry.register(CommandToken::Type, Arc::new(Type::new(paths.clone())));
        registry.register(CommandToken::Pwd, Arc::new(Pwd));

        Self {
            builtins: registry,
            paths,
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

            let command = self.builtins.execute(buffer.trim());

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
                },
            };

            io::stdout().flush().unwrap();
        }
    }
}
