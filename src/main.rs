use crate::executable::repl::Repl;

pub(crate) mod commands;
pub(crate) mod exceptions;
pub(crate) mod executable;
pub mod external;
pub(crate) mod parser;
pub(crate) mod port;
pub mod shell;

fn main() {
    Repl::new().spawn().unwrap();
}
