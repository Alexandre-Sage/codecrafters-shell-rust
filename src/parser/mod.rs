use std::{char, usize};

use crate::exceptions::commands::CommandError;

const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum QuoteType {
    Single,
    Double,
}

impl TryFrom<char> for QuoteType {
    type Error = CommandError;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            SINGLE_QUOTE => Ok(Self::Single),
            DOUBLE_QUOTE => Ok(Self::Double),
            _ => Err(CommandError::Unknown("Not a quote".to_owned())),
        }
    }
}

impl QuotePosition {
    fn start(&self) -> &usize {
        match self {
            Self::SingleQuote(start, _) | Self::DoubleQuote(start, _) => start,
        }
    }

    fn end(&self) -> &usize {
        match self {
            Self::DoubleQuote(_, end) | Self::SingleQuote(_, end) => end,
        }
    }
}

pub struct InputParser;

#[derive(Debug)]
pub struct ParsedCommand(String, Vec<String>);

impl ParsedCommand {
    pub fn new(command: &str, args: Vec<String>) -> Self {
        Self(command.to_owned(), args)
    }

    pub fn args(&self) -> &[String] {
        &self.1
    }

    pub fn command(&self) -> &str {
        &self.0
    }

    fn from_raw((command, args): (&str, &str)) -> Self {
        let args = args
            .split_whitespace()
            .map(|item| item.to_owned())
            .collect();

        let command = command
            .chars()
            .filter(|char| *char != SINGLE_QUOTE && *char != DOUBLE_QUOTE)
            .collect();

        Self(command, args)
    }
}

impl InputParser {
    pub fn new() -> Self {
        Self
    }

    fn quote_positions(&self, args: &str) -> Result<Vec<QuotePosition>, CommandError> {
        let mut opening_quote: Option<(QuoteType, usize)> = None;
        let mut quote_positions: Vec<QuotePosition> = Vec::new();

        for (idx, char) in args.chars().enumerate() {
            if char == SINGLE_QUOTE || char == DOUBLE_QUOTE {
                let quote_type = QuoteType::try_from(char)?;

                match opening_quote {
                    Some((quote, start)) => {
                        if quote == quote_type {
                            let quote_pos = QuotePosition::from((quote, (start, idx)));
                            quote_positions.push(quote_pos);
                            opening_quote = None;
                        }
                    }
                    None => {
                        opening_quote.replace((quote_type, idx));
                    }
                }
            }
        }

        if opening_quote.is_some() {
            return Err(CommandError::MissingClosingQuote);
        }

        Ok(quote_positions)
    }

    pub fn parse_quote(&self, quote_positions: &[QuotePosition], args: &str) -> Vec<String> {
        let quote_positions_filtered: Vec<_> = quote_positions
            .iter()
            .filter(|position| position.start() + 1 != *position.end())
            .collect();

        let mut parsed_args: Vec<String> = vec![];
        let mut tmp: Vec<char> = vec![];
        for (idx, char) in args.chars().enumerate() {
            // Use strict inequalities (> and <) to exclude quote boundaries                                                                                                          │
            // This means opening/closing quotes are NOT "inside" the range                                                                                                           │
            let maybe_quote_pos = quote_positions_filtered
                .iter()
                .find(|pos| idx > *pos.start() && idx < *pos.end());

            if maybe_quote_pos.is_some() {
                tmp.push(char);
                continue;
            }

            if char == ' ' {
                if !tmp.is_empty() {
                    parsed_args.push(tmp.iter().collect());
                    tmp.clear();
                }
                continue;
            }

            if char != SINGLE_QUOTE && char != DOUBLE_QUOTE {
                tmp.push(char);
            }

            if idx == args.len() - 1 && !tmp.is_empty() {
                parsed_args.push(tmp.iter().collect());
            }
        }
        parsed_args
    }

    pub fn parse(&self, input: &str) -> Result<ParsedCommand, CommandError> {
        let (command, args) = input.split_once(" ").unwrap_or((input, ""));

        if args.contains(SINGLE_QUOTE) || args.contains(DOUBLE_QUOTE) {
            let quote_positions = self.quote_positions(args)?;
            let parsed_args = self.parse_quote(&quote_positions, args);
            return Ok(ParsedCommand::new(command, parsed_args));
        }

        Ok(ParsedCommand::from_raw((command, args)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Happy Path Tests - Single Quotes
    // ========================================================================

    #[test]
    fn parse_simple_single_quoted_string() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello world'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
    }

    #[test]
    fn parse_single_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello    world'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
    }

    #[test]
    fn parse_multiple_single_quoted_arguments() {
        let parser = InputParser::new();
        let result = parser.parse("cat '/tmp/file1' '/tmp/file2'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
    }

    #[test]
    fn parse_adjacent_single_quoted_strings_concatenate() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello''world'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
    }

    #[test]
    fn parse_empty_single_quotes_ignored() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello''world");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
    }

    #[test]
    fn parse_single_quotes_with_special_characters() {
        let parser = InputParser::new();
        let result = parser.parse("echo '$HOME * ? [] | & ;'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // All special chars treated literally
        assert_eq!(parsed.args(), &["$HOME * ? [] | & ;"]);
    }

    // ========================================================================
    // Edge Case Tests - Single Quotes
    // ========================================================================

    #[test]
    fn parse_single_quote_with_spaces_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat '/tmp/file name with spaces'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
    }

    #[test]
    fn parse_mixed_quoted_and_unquoted() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello 'world test' foo");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
    }

    #[test]
    fn parse_unclosed_single_quote_returns_error() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello world");

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert_eq!(err_msg, CommandError::MissingClosingQuote);
    }

    #[test]
    fn parse_only_quotes_no_command() {
        let parser = InputParser::new();
        let result = parser.parse("'hello'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "hello");
        assert_eq!(parsed.args().len(), 0);
    }

    // ========================================================================
    // Happy Path Tests - Double Quotes
    // ========================================================================

    #[test]
    fn parse_simple_double_quoted_string() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello world\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
    }

    #[test]
    fn parse_double_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello    world\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
    }

    #[test]
    fn parse_multiple_double_quoted_arguments() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file1\" \"/tmp/file2\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
    }

    #[test]
    fn parse_adjacent_double_quoted_strings_concatenate() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\"\"world\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
    }

    #[test]
    fn parse_empty_double_quotes_ignored() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\"\"world");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
    }

    #[test]
    fn parse_double_quote_with_spaces_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file name with spaces\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
    }

    #[test]
    fn parse_mixed_double_quoted_and_unquoted() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello \"world test\" foo");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
    }

    #[test]
    fn parse_unclosed_double_quote_returns_error() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello world");

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert_eq!(err_msg, CommandError::MissingClosingQuote);
    }

    #[test]
    fn parse_double_quotes_command_name() {
        let parser = InputParser::new();
        let result = parser.parse("\"echo\" hello");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);
    }

    #[test]
    fn parse_double_quotes_only_command() {
        let parser = InputParser::new();
        let result = parser.parse("\"hello\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "hello");
        assert_eq!(parsed.args().len(), 0);
    }

    #[test]
    fn parse_mixed_single_and_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'single' \"double\" plain");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["single", "double", "plain"]);
    }

    #[test]
    fn parse_double_quotes_with_single_quote_inside() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello'world\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello'world"]);
    }

    #[test]
    fn parse_single_quotes_with_double_quote_inside() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello\"world'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\"world"]);
    }

    #[test]
    fn parse_alternating_single_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"a\"'b'\"c\"'d'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["abcd"]);
    }
}
