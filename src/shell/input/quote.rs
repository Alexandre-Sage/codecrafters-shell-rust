use crate::{
    exceptions::commands::ShellError,
    shell::input::commons::{DOUBLE_QUOTE, SINGLE_QUOTE},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QuoteType {
    Single,
    Double,
}

impl TryFrom<char> for QuoteType {
    type Error = ShellError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            SINGLE_QUOTE => Ok(Self::Single),
            DOUBLE_QUOTE => Ok(Self::Double),
            _ => Err(ShellError::Uncontroled("Not a quote".to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum QuotePosition {
    SingleQuote(usize, usize),
    DoubleQuote(usize, usize),
}

impl From<(QuoteType, (usize, usize))> for QuotePosition {
    fn from((quote_type, (start, end)): (QuoteType, (usize, usize))) -> Self {
        match quote_type {
            QuoteType::Single => Self::SingleQuote(start, end),
            QuoteType::Double => Self::DoubleQuote(start, end),
        }
    }
}

impl QuotePosition {
    pub fn start(&self) -> &usize {
        match self {
            Self::SingleQuote(start, _) | Self::DoubleQuote(start, _) => start,
        }
    }

    pub fn end(&self) -> &usize {
        match self {
            Self::DoubleQuote(_, end) | Self::SingleQuote(_, end) => end,
        }
    }

    pub fn is_doulbe_quote(&self) -> bool {
        matches!(self, QuotePosition::DoubleQuote(_, _))
    }
}
