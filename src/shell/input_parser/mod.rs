pub mod commons;
pub mod quote;
pub mod redirection_context;

use std::{char, path::PathBuf, usize};

use crate::{
    exceptions::commands::CommandError,
    shell::{
        input_parser::{
            commons::{BACK_SLASH, DOUBLE_QUOTE, SINGLE_QUOTE},
            quote::{QuotePosition, QuoteType},
            redirection_context::{RedirectionContext, RedirectionType},
        },
        redirection,
    },
};

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
        let c = args.chars();
        for (idx, char) in c.enumerate() {
            if !quote_positions_filtered.is_empty() {
                // Use strict inequalities (> and <) to exclude quote boundaries                                                                                                          │
                // This means opening/closing quotes are NOT "inside" the range                                                                                                           │
                let maybe_quote_pos = quote_positions_filtered
                    .iter()
                    .find(|pos| idx > *pos.start() && idx < *pos.end());

                if let Some(quote_pos) = maybe_quote_pos {
                    if char == BACK_SLASH && quote_pos.is_doulbe_quote() && !escape_next {
                        escape_next = true;
                        continue;
                    }

                    if escape_next {
                        if char == DOUBLE_QUOTE || char == BACK_SLASH {
                            // current_arg.push(BACK_SLASH);
                            current_arg.push(char);
                        } else {
                            current_arg.push(BACK_SLASH);
                            current_arg.push(char);
                        }
                        escape_next = false;
                        continue;
                    }
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

    fn parse_redirection(
        &self,
        args: &mut Vec<String>,
    ) -> Result<Option<RedirectionContext>, CommandError> {
        let maybe_redirection = args.iter().enumerate().find_map(|(idx, part)| {
            RedirectionType::try_from(part.as_str())
                .map(|res| (idx, res))
                .ok()
        });
        if let Some((pos, redirection)) = maybe_redirection {
            if pos + 1 >= args.len() {
                return Err(CommandError::Unknown(
                    "Missing filename after redirection operator".to_string(),
                ));
            }

            let parts: Vec<_> = args.drain(pos..pos + 2).collect();
            let redirection = RedirectionContext::new(PathBuf::from(&parts[1]), redirection);

            return Ok(Some(redirection));
        }
        Ok(None)
    }

    pub fn parse(
        &self,
        input: &str,
    ) -> Result<(ParsedCommand, Option<RedirectionContext>), CommandError> {
        let quote_positions = self.quote_positions(input)?;
        let mut parsed_args = self.parse_args(&quote_positions, input);
        let maybe_redirection = self.parse_redirection(&mut parsed_args)?;
        let command = &parsed_args[0];

        Ok((
            ParsedCommand::new(command, parsed_args[1..].to_vec()),
            maybe_redirection,
        ))
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
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello    world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_single_quoted_arguments() {
        let parser = InputParser::new();
        let result = parser.parse("cat '/tmp/file1' '/tmp/file2'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_adjacent_single_quoted_strings_concatenate() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello''world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_empty_single_quotes_ignored() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello''world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_with_special_characters() {
        let parser = InputParser::new();
        let result = parser.parse("echo '$HOME * ? [] | & ;'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // All special chars treated literally
        assert_eq!(parsed.args(), &["$HOME * ? [] | & ;"]);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Edge Case Tests - Single Quotes
    // ========================================================================

    #[test]
    fn parse_single_quote_with_spaces_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat '/tmp/file name with spaces'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_quoted_and_unquoted() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello 'world test' foo");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
        assert!(redirection.is_none());
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
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "hello");
        assert_eq!(parsed.args().len(), 0);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Happy Path Tests - Double Quotes
    // ========================================================================

    #[test]
    fn parse_simple_double_quoted_string() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello    world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_double_quoted_arguments() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file1\" \"/tmp/file2\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_adjacent_double_quoted_strings_concatenate() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\"\"world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_empty_double_quotes_ignored() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\"\"world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quote_with_spaces_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file name with spaces\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_double_quoted_and_unquoted() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello \"world test\" foo");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
        assert!(redirection.is_none());
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
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_only_command() {
        let parser = InputParser::new();
        let result = parser.parse("\"hello\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "hello");
        assert_eq!(parsed.args().len(), 0);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_single_and_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'single' \"double\" plain");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["single", "double", "plain"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_with_single_quote_inside() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello'world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello'world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_with_double_quote_inside() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello\"world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\"world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_alternating_single_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"a\"'b'\"c\"'d'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["abcd"]);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Happy Path Tests - Backslash Escaping
    // ========================================================================

    #[test]
    fn parse_backslash_escapes_space() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_escaped_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo world\\ \\ \\ \\ \\ \\ script");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["world      script"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_inside_double_quotes_preserved() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"before\\   after\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["before\\   after"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_special_chars() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\$HOME");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["$HOME"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\\"hello\\\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["\"hello\""]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_single_quote() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\'hello\\'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["'hello'"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_at_end_of_argument() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ ");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello "]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_backslash_produces_single() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\\\world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\\world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_with_regular_char() {
        let parser = InputParser::new();
        let result = parser.parse("echo \\a\\b\\c");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["abc"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_escaped_and_unescaped_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\ world foo");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world", "foo"]);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Edge Case Tests - Backslash Escaping
    // ========================================================================

    #[test]
    fn parse_backslash_in_filename() {
        let parser = InputParser::new();
        let result = parser.parse("cat /tmp/file\\ name");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name"]);
        assert!(redirection.is_none());
    }

    // #[test]
    // fn parse_double_backslash_inside_double_quotes() {
    //     let parser = InputParser::new();
    //     let result = parser.parse("cat \"/tmp/file\\\\name\"");
    //
    //     assert!(result.is_ok());
    //     let parsed = result.unwrap();
    //     assert_eq!(parsed.command(), "cat");
    //     assert_eq!(parsed.args(), &["/tmp/file\\\\name"]);
    // }

    #[test]
    fn parse_backslash_space_inside_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/file\\ name\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file\\ name"]);
        assert!(redirection.is_none());
    }

    // #[test]
    // fn parse_multiple_files_with_backslashes_in_quotes() {
    //     let parser = InputParser::new();
    //     let result = parser.parse("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\"");
    //
    //     assert!(result.is_ok());
    //     let parsed = result.unwrap();
    //     assert_eq!(parsed.command(), "cat");
    //     assert_eq!(parsed.args(), &["/tmp/file\\\\name", "/tmp/file\\ name"]);
    // }

    #[test]
    fn parse_backslash_does_not_escape_inside_single_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo 'hello\\ world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Inside single quotes, backslash is literal
        assert_eq!(parsed.args(), &["hello\\ world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_quotes_and_backslashes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"quoted\"\\ unquoted");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["quoted unquoted"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_newline_continuation() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\nworld");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Backslash followed by 'n' produces literal 'n'
        assert_eq!(parsed.args(), &["hellonworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_command_with_escaped_backslash() {
        let parser = InputParser::new();
        let result = parser.parse("echo C:\\\\Users\\\\file");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["C:\\Users\\file"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_trailing_backslash_escapes_nothing() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello\\");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Trailing backslash with nothing to escape
        // This might be implementation-specific, but typically treated as literal
        assert_eq!(parsed.args(), &["hello"]);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Backslash Escaping Inside Double Quotes
    // ========================================================================

    #[test]
    fn parse_double_backslash_inside_double_quotes_produces_single() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\\\\world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Inside double quotes, \\ should produce single \
        assert_eq!(parsed.args(), &["hello\\world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_escaped_double_quote_inside_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\\\"world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Inside double quotes, \" should produce literal "
        assert_eq!(parsed.args(), &["hello\"world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quote_with_escaped_quote_at_boundaries() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"\\\"start\" \"end\\\"\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // \" at start and end of double quoted strings
        assert_eq!(parsed.args(), &["\"start", "end\""]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_escaped_backslashes_in_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"\\\\\\\\\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // Four backslashes become two
        assert_eq!(parsed.args(), &["\\\\"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_with_non_escapable_char_in_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\\nworld\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // \n inside double quotes: backslash followed by 'n' (not escape sequence)
        // According to bash, only \, ", $, `, newline are escapable
        // So \n should remain as literal \n
        assert_eq!(parsed.args(), &["hello\\nworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_escapes_in_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"A \\\\ escapes itself\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // \\ produces single \
        assert_eq!(parsed.args(), &["A \\ escapes itself"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_with_escaped_quote_in_middle() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"A \\\" inside double quotes\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // \" produces literal "
        assert_eq!(parsed.args(), &["A \" inside double quotes"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_codecrafter_test_case() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"script'hello'\\\\'example\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // script'hello' (single quotes literal)
        // \\ (produces single \)
        // ' (literal single quote)
        // example
        assert_eq!(parsed.args(), &["script'hello'\\'example"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_escaped_quote_allows_concatenation() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"hello\\\"insidequotes\"script\\\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // "hello\"insidequotes" produces: hello"insidequotes
        // script\" (outside quotes) produces: script"
        // Concatenated: hello"insidequotesscript"
        assert_eq!(parsed.args(), &["hello\"insidequotesscript\""]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_backslash_before_regular_char_in_double_quotes() {
        let parser = InputParser::new();
        let result = parser.parse("echo \"test\\avalue\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // \a is not escapable in double quotes, so backslash is literal
        assert_eq!(parsed.args(), &["test\\avalue"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_filename_with_escaped_backslash_and_quote() {
        let parser = InputParser::new();
        let result = parser.parse("cat \"/tmp/\\\"f\\\\93\\\"\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        // \" produces ", \\ produces \
        // Result: /tmp/"f\93"
        assert_eq!(parsed.args(), &["/tmp/\"f\\93\""]);
        assert!(redirection.is_none());
    }

    // ========================================================================
    // Output Redirection Tests
    // ========================================================================

    #[test]
    fn parse_simple_output_redirection() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello > output.txt");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        // Check redirection was detected
        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("output.txt"));
        assert_eq!(
            redir.redirection_type,
            RedirectionType::Output(redirection_context::RedirectionChannel::Stdout)
        );
    }

    #[test]
    fn parse_redirection_with_spaces() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello   >   output.txt");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("output.txt"));
    }

    #[test]
    fn parse_redirection_with_quoted_filename() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello > \"file with spaces.txt\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("file with spaces.txt"));
    }

    #[test]
    fn parse_command_without_redirection() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_redirection_at_end() {
        let parser = InputParser::new();
        let result = parser.parse("cat file1 file2 > output.txt");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["file1", "file2"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("output.txt"));
    }

    #[test]
    fn parse_redirection_with_single_quoted_filename() {
        let parser = InputParser::new();
        let result = parser.parse("echo test > 'output file.txt'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["test"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("output file.txt"));
    }

    #[test]
    fn parse_redirection_with_path() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello > /tmp/output.txt");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, PathBuf::from("/tmp/output.txt"));
    }

    #[test]
    fn parse_quoted_command_with_redirection() {
        let parser = InputParser::new();
        let result = parser.parse("\"echo\" hello > out.txt");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);
        assert!(redirection.is_some());
    }

    #[test]
    fn parse_redirection_in_middle_is_not_detected() {
        let parser = InputParser::new();
        // The > inside quotes should not be treated as redirection
        let result = parser.parse("echo 'hello > world' test");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello > world", "test"]);
        // > is inside quotes, so no redirection
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_words_after_redirection_operator() {
        let parser = InputParser::new();
        // Should only take the first word/quoted string as filename
        let result = parser.parse("echo hello > output.txt extra");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        // "extra" should probably cause an error or be ignored
        // For now, we expect it in args
        assert!(parsed.args().contains(&"hello".to_string()));
        assert!(redirection.is_some());
    }

    // ========================================================================
    // Edge Case Tests - Redirection Errors
    // ========================================================================

    #[test]
    fn parse_redirection_without_filename_returns_error() {
        let parser = InputParser::new();
        let result = parser.parse("echo hello >");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, CommandError::Unknown(_)),
            "Expected Unknown error for missing filename after >"
        );
    }
}
