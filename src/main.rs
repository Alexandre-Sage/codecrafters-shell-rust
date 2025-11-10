pub(crate) mod exceptions;

#[allow(unused_imports)]
use std::io::{self, Write};

use crate::exceptions::{application::ApplicationError, commands::CommandError};

fn run(command: &str) -> Result<(), ApplicationError> {
    return Err(ApplicationError::CommandError(
        CommandError::CommandNotFound(command.to_owned()),
    ));
}

struct Repl;

impl Repl {
    fn prompt(&self) -> Result<(), io::Error> {
        print!("$ ");

        io::stdout().flush()
    }

    fn spawn(&self) {
        loop {
            self.prompt().unwrap();

            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();

            match run(&buffer.trim()) {
                Err(err) => print!("{}\n", err.to_string()),
                Ok(_) => todo!(),
            };
            io::stdout().flush().unwrap();
        }
    }
}

fn main() {
    Repl.spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_not_found_error() {
        let result = run("xyz").unwrap_err();

        assert_eq!(
            result,
            ApplicationError::CommandError(CommandError::CommandNotFound("xyz".to_owned()))
        )
    }
}
