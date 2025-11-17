use std::{char, usize};

use crate::exceptions::commands::CommandError;

pub struct InputParser;

#[derive(Debug)]
pub struct ParsedCommand(String, Vec<String>);

impl ParsedCommand {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self(command, args)
    }

    pub fn args(&self) -> &[String] {
        &self.1
    }

    pub fn command(&self) -> &str {
        &self.0
    }
}

impl InputParser {
    pub fn new() -> Self {
        Self
    }

    fn quote_pos(&self, args: &str) -> Result<Vec<(usize, usize)>, CommandError> {
        let quote_pos = args
            .chars()
            .enumerate()
            .filter(|(_, char)| *char == '\'')
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();

        if quote_pos.len() % 2 != 0 {
            return Err(CommandError::MissingClosingSingleQuote);
        }

        let quote_pos_merged: Vec<(usize, usize)> = quote_pos
            .chunks(2)
            .filter(|item| item[0] + 1 != item[1])
            .enumerate()
            .fold(vec![], |mut acc, (idx, cur)| {
                if !acc.is_empty() && acc[idx - 1].1 == cur[0] - 1 {
                    acc[idx - 1] = (acc[idx - 1].0, cur[1]);
                    return acc;
                }

                acc.push((cur[0], cur[1]));
                acc
            });

        Ok(quote_pos_merged)
    }

    pub fn parse(&self, input: &str) -> Result<ParsedCommand, CommandError> {
        let (command, args) = input.split_once(" ").unwrap_or((input, ""));

        if args.contains("'") {
            let quote_pos_merged = self.quote_pos(args)?;
            let mut parsed_args: Vec<String> = vec![];
            let mut tmp: Vec<char> = vec![];

            for (idx, char) in args.chars().enumerate() {
                let is_between_quote = quote_pos_merged
                    .iter()
                    .any(|pos| idx > pos.0 && idx < pos.1);
                let is_closing_quote = quote_pos_merged.iter().any(|pos| pos.1 == idx);

                if (char == ' ' && !is_between_quote) || is_closing_quote {
                    if !tmp.is_empty() {
                        parsed_args.push(tmp.iter().collect());
                        tmp.clear();
                    }
                    continue;
                }

                if char != '\'' {
                    tmp.push(char);
                }

                if idx == args.len() - 1 {
                    if !tmp.is_empty() {
                        parsed_args.push(tmp.iter().collect());
                    }
                }
            }

            return Ok(ParsedCommand::new(command.to_owned(), parsed_args));
        }

        let args = args
            .split_whitespace()
            .map(|item| item.to_owned())
            .collect();

        let command = command.chars().filter(|char| *char != '\'').collect();

        Ok(ParsedCommand::new(command, args))
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
    // Edge Case Tests
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
        assert_eq!(err_msg, CommandError::MissingClosingSingleQuote);
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
}
