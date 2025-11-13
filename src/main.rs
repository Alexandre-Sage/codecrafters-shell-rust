use crate::executable::repl::Repl;

pub(crate) mod commands;
pub(crate) mod exceptions;
pub(crate) mod executable;
pub(crate) mod port;
pub mod shell;

fn main() {
    Repl::new().spawn().unwrap();
}
