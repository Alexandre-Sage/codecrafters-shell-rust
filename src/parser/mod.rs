use std::{char, usize};

use crate::exceptions::commands::CommandError;

const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const BACK_SLASH: char = '\\';

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
        let command = command
            .chars()
            .filter(|char| *char != SINGLE_QUOTE && *char != DOUBLE_QUOTE)
            .collect();

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

    fn quote_positions(&self, args: &str) -> Result<Vec<QuotePosition>, CommandError> {
        if !args.contains(SINGLE_QUOTE) && !args.contains(DOUBLE_QUOTE) {
            return Ok(vec![]);
        }

        let mut opening_quote: Option<(QuoteType, usize)> = None;
        let mut escape_next_quote = false;
        let mut quote_positions: Vec<QuotePosition> = Vec::new();

        for (idx, char) in args.chars().enumerate() {
            if char == BACK_SLASH && !escape_next_quote {
                escape_next_quote = true;
                continue;
            }

            if (char == SINGLE_QUOTE || char == DOUBLE_QUOTE) && !escape_next_quote {
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
            escape_next_quote = false
        }

        if opening_quote.is_some() {
            return Err(CommandError::MissingClosingQuote);
        }

        Ok(quote_positions)
    }

    pub fn parse_args(&self, quote_positions: &[QuotePosition], args: &str) -> Vec<String> {
        let quote_positions_filtered: Vec<_> = quote_positions
            .iter()
            .filter(|position| position.start() + 1 != *position.end())
            .collect();

        let mut parsed_args: Vec<String> = vec![];
        let mut current_arg: Vec<char> = vec![];
        let mut escape_next = false;
        for (idx, char) in args.chars().enumerate() {
            if !quote_positions_filtered.is_empty() {
                // Use strict inequalities (> and <) to exclude quote boundaries                                                                                                          │
                // This means opening/closing quotes are NOT "inside" the range                                                                                                           │
                let maybe_quote_pos = quote_positions_filtered
                    .iter()
                    .find(|pos| idx > *pos.start() && idx < *pos.end());

                if maybe_quote_pos.is_some() {
                    current_arg.push(char);
                    continue;
                }
            }

            if char == BACK_SLASH && !escape_next && idx != args.len() - 1 {
                escape_next = true;
                continue;
            }

            if char == ' ' && !escape_next {
                if !current_arg.is_empty() {
                    parsed_args.push(current_arg.iter().collect());
                    current_arg.clear();
                }
                continue;
            }

            if (char != SINGLE_QUOTE && char != DOUBLE_QUOTE && char != BACK_SLASH) || escape_next {
                current_arg.push(char);
            }

            if idx == args.len() - 1 && !current_arg.is_empty() {
                parsed_args.push(current_arg.iter().collect());
            }
            escape_next = false
        }
        parsed_args
    }

    pub fn parse(&self, input: &str) -> Result<ParsedCommand, CommandError> {
        let (command, args) = input.split_once(" ").unwrap_or((input, ""));

        let quote_positions = self.quote_positions(args)?;
        let parsed_args = self.parse_args(&quote_positions, args);
        return Ok(ParsedCommand::new(command, parsed_args));
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

    // ========================================================================
    // Happy Path Tests - Backslash Escaping
    // ========================================================================

    #[test]
    fn parse_backslash_escapes_space() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ world");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
    }

    #[test]
    fn parse_multiple_escaped_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo world\\ \\ \\ \\ \\ \\ script");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["world      script"]);
    }

    #[test]
    fn parse_backslash_inside_double_quotes_preserved() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"before\\   after\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["before\\   after"]);
    }

    #[test]
    fn parse_backslash_escapes_special_chars() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\$HOME");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["$HOME"]);
    }

    #[test]
    fn parse_backslash_escapes_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\\"hello\\\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["\"hello\""]);
    }

    #[test]
    fn parse_backslash_escapes_single_quote() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\'hello\\'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["'hello'"]);
    }

    #[test]
    fn parse_backslash_at_end_of_argument() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ ");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello "]);
    }

    #[test]
    fn parse_double_backslash_produces_single() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\\\world");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\\world"]);
    }

    #[test]
    fn parse_backslash_with_regular_char() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\a\\b\\c");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["abc"]);
    }

    #[test]
    fn parse_mixed_escaped_and_unescaped_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ world foo");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world", "foo"]);
    }

    // ========================================================================
    // Edge Case Tests - Backslash Escaping
    // ========================================================================

    #[test]
    fn parse_backslash_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat /tmp/file\\ name");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name"]);
    }

    #[test]
    fn parse_double_backslash_inside_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file\\\\name\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file\\\\name"]);
    }

    #[test]
    fn parse_backslash_space_inside_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file\\ name\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file\\ name"]);
    }

    #[test]
    fn parse_multiple_files_with_backslashes_in_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\"");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file\\\\name", "/tmp/file\\ name"]);
    }

    #[test]
    fn parse_backslash_does_not_escape_inside_single_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello\\ world'");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Inside single quotes, backslash is literal
        assert_eq!(parsed.args(), &["hello\\ world"]);
    }

    #[test]
    fn parse_mixed_quotes_and_backslashes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"quoted\"\\ unquoted");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["quoted unquoted"]);
    }

    #[test]
    fn parse_backslash_newline_continuation() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\nworld");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Backslash followed by 'n' produces literal 'n'
        assert_eq!(parsed.args(), &["hellonworld"]);
    }

    #[test]
    fn parse_command_with_escaped_backslash() {
        let parser = InputParser::new();
        let result = parser.parse("echo C:\\\\Users\\\\file");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["C:\\Users\\file"]);
    }

    #[test]
    fn parse_trailing_backslash_escapes_nothing() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\");

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Trailing backslash with nothing to escape
        // This might be implementation-specific, but typically treated as literal
        assert_eq!(parsed.args(), &["hello"]);
    }
}
