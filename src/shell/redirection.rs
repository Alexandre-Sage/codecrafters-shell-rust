use std::{path::PathBuf, str::FromStr};

use crate::exceptions::commands::CommandError;

#[derive(Debug)]
pub enum RedirectionType {
    Output,
}

impl TryFrom<char> for RedirectionType {
    type Error = CommandError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct RedirectionContext {
    path: PathBuf,
    mode: i8,
    redirection_type: RedirectionType,
}

pub struct RedirectionManager;

impl RedirectionManager {
    pub fn write_to_file(&self, context: RedirectionContext) {
        todo!()
    }
}
