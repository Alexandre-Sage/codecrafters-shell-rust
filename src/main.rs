#[allow(unused_imports)]
use std::io::{self, Write};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
enum CommandError<'a> {
    #[error("{0}: command not found")]
    CommandNotFound(&'a str),
}

enum Command {}

fn run(command: &str) -> Result<(), CommandError> {
    return Err(CommandError::CommandNotFound(command));
}

fn main() {
    print!("$ ");

    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();

    match run(&buffer.trim()) {
        Err(err) => print!("{}", err.to_string()),
        Ok(_) => todo!(),
    };
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_not_found_error() {
        let result = run("xyz").unwrap_err();

        assert_eq!(result, CommandError::CommandNotFound("xyz"))
    }
}
