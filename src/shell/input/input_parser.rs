use std::{char, path::PathBuf, sync::Arc, usize};

use crate::exceptions::commands::ShellError;
use crate::shell::file::FileManager;
use crate::shell::input::commons::{BACK_SLASH, DOUBLE_QUOTE, SINGLE_QUOTE};
use crate::shell::input::quote::{QuotePosition, QuoteType};
use crate::shell::input::redirection_context::{RedirectionContext, RedirectionType};

#[derive(Debug, Default)]
struct ParserState<'a> {
    escape_next: bool,
    current_arg: Vec<char>,
    parsed_args: Vec<String>,
    quote_position: Option<&'a QuotePosition>,
}

impl<'a> ParserState<'a> {
    fn push_litteral(&mut self, char: char) {
        self.current_arg.push(char);
    }

    fn escape_next(&mut self) {
        self.escape_next = true
    }

    fn reset_escape(&mut self) {
        self.escape_next = false
    }

    fn finalize_arg(&mut self) {
        if !self.current_arg.is_empty() {
            self.parsed_args.push(self.current_arg.iter().collect());
            self.current_arg.clear();
        }
    }

    fn set_quote_postion(&mut self, quote_position: &'a QuotePosition) {
        self.quote_position.replace(quote_position);
    }
}

pub struct InputParser {
    file_manager: Arc<FileManager>,
}

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
    pub fn new(file_manager: Arc<FileManager>) -> Self {
        Self { file_manager }
    }

    fn quote_positions(&self, args: &str) -> Result<Vec<QuotePosition>, ShellError> {
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
            return Err(ShellError::MissingClosingQuote);
        }

        let quote_positions_filtered: Vec<_> = quote_positions
            .into_iter()
            .filter(|position| position.start() + 1 != *position.end())
            .collect();

        Ok(quote_positions_filtered)
    }

    fn should_escape_next(&self, state: &mut ParserState, char: char) {
        if char != BACK_SLASH {
            return;
        }

        if let Some(quote_position) = state.quote_position {
            if quote_position.is_doulbe_quote() && !state.escape_next {
                state.escape_next()
            }
        }

        if !state.escape_next {
            state.escape_next()
        }
    }

    fn handle_quote(&self, parser_state: &mut ParserState, char: char) {
        if char == BACK_SLASH
            // TODO REMOVE UNWRAP IN LAST REFACTO ITERATION
            && parser_state.quote_position.unwrap().is_doulbe_quote()
            && !parser_state.escape_next
        {
            parser_state.escape_next();
            return;
        }

        if parser_state.escape_next {
            if char == DOUBLE_QUOTE || char == BACK_SLASH {
                parser_state.push_litteral(char);
            } else {
                parser_state.push_litteral(BACK_SLASH);
                parser_state.push_litteral(char);
            }
            parser_state.reset_escape();
            return;
        }
        parser_state.push_litteral(char);
        return;
    }

    fn handle_regular(&self, parser_state: &mut ParserState, args: &str, idx: usize, char: char) {
        if char == BACK_SLASH && !parser_state.escape_next && idx != args.len() - 1 {
            parser_state.escape_next();
            return;
        }

        if char == ' ' && !parser_state.escape_next {
            parser_state.finalize_arg();
            return;
        }

        if (char != SINGLE_QUOTE && char != DOUBLE_QUOTE && char != BACK_SLASH)
            || parser_state.escape_next
        {
            parser_state.push_litteral(char);
            parser_state.reset_escape();
        }
    }

    pub fn parse_args(
        &self,
        quote_positions_filtered: &[QuotePosition],
        args: &str,
    ) -> Vec<String> {
        let mut parser_state = ParserState::default();

        let c = args.chars();
        for (idx, char) in c.enumerate() {
            if !quote_positions_filtered.is_empty() {
                // Use strict inequalities (> and <) to exclude quote boundaries                                                                                                          │
                // This means opening/closing quotes are NOT "inside" the range                                                                                                           │
                let maybe_quote_pos = quote_positions_filtered
                    .iter()
                    .find(|pos| idx > *pos.start() && idx < *pos.end());

                if let Some(quote_pos) = maybe_quote_pos {
                    parser_state.set_quote_postion(&quote_pos);
                    self.handle_quote(&mut parser_state, char);
                    continue;
                }
            }

            self.handle_regular(&mut parser_state, args, idx, char);
        }

        parser_state.finalize_arg();

        parser_state.parsed_args
    }

    fn parse_redirection(
        &self,
        args: &mut Vec<String>,
    ) -> Result<Option<RedirectionContext>, ShellError> {
        let maybe_redirection = args.iter().enumerate().find_map(|(idx, part)| {
            RedirectionType::try_from(part.as_str())
                .map(|res| (idx, res))
                .ok()
        });
        if let Some((pos, redirection)) = maybe_redirection {
            if pos + 1 >= args.len() {
                return Err(ShellError::Uncontroled(
                    "Missing filename after redirection operator".to_string(),
                ));
            }

            let parts: Vec<_> = args.drain(pos..pos + 2).collect();
            let path = PathBuf::from(&parts[1]);

            self.file_manager.parent_dir_exist(&path)?;
            self.file_manager.create_file_if_no_exist(&path)?;

            let redirection = RedirectionContext::new(path, redirection);

            return Ok(Some(redirection));
        }
        Ok(None)
    }

    pub fn parse(
        &self,
        input: &str,
    ) -> Result<(ParsedCommand, Option<RedirectionContext>), ShellError> {
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

    use crate::shell::input::redirection_context::{self, RedirectionChannel};

    use super::*;

    // ========================================================================
    // Happy Path Tests - Single Quotes
    // ========================================================================

    #[test]
    fn parse_simple_single_quoted_string() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello    world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_single_quoted_arguments() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("cat '/tmp/file1' '/tmp/file2'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_adjacent_single_quoted_strings_concatenate() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello''world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_empty_single_quotes_ignored() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello''world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_with_special_characters() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("cat '/tmp/file name with spaces'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_quoted_and_unquoted() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello 'world test' foo");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_unclosed_single_quote_returns_error() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello world");

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert_eq!(
            err_msg.to_string(),
            ShellError::MissingClosingQuote.to_string()
        );
    }

    #[test]
    fn parse_only_quotes_no_command() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"hello world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_preserve_multiple_spaces() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"hello    world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello    world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_double_quoted_arguments() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("cat \"/tmp/file1\" \"/tmp/file2\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file1", "/tmp/file2"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_adjacent_double_quoted_strings_concatenate() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"hello\"\"world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_empty_double_quotes_ignored() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello\"\"world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["helloworld"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quote_with_spaces_in_filename() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("cat \"/tmp/file name with spaces\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["/tmp/file name with spaces"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_double_quoted_and_unquoted() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello \"world test\" foo");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world test", "foo"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_unclosed_double_quote_returns_error() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"hello world");

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert_eq!(err_msg, ShellError::MissingClosingQuote);
    }

    #[test]
    fn parse_double_quotes_command_name() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("\"echo\" hello");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_only_command() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("\"hello\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "hello");
        assert_eq!(parsed.args().len(), 0);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_single_and_double_quotes() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'single' \"double\" plain");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["single", "double", "plain"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_quotes_with_single_quote_inside() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"hello'world\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello'world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_single_quotes_with_double_quote_inside() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello\"world'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\"world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_alternating_single_double_quotes() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello\\ world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_escaped_spaces() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo world\\ \\ \\ \\ \\ \\ script");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["world      script"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_inside_double_quotes_preserved() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"before\\   after\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["before\\   after"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_special_chars() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \\$HOME");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["$HOME"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_quotes() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \\\"hello\\\"");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["\"hello\""]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_escapes_single_quote() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \\'hello\\'");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["'hello'"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_at_end_of_argument() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello\\ ");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello "]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_double_backslash_produces_single() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello\\\\world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello\\world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_with_regular_char() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \\a\\b\\c");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["abc"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_mixed_escaped_and_unescaped_spaces() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo \"quoted\"\\ unquoted");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["quoted unquoted"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_backslash_newline_continuation() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo C:\\\\Users\\\\file");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["C:\\Users\\file"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_trailing_backslash_escapes_nothing() {
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let parser = InputParser::new(Arc::new(FileManager));
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
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello > {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        // Check redirection was detected
        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert_eq!(
            redir.redirection_type,
            RedirectionType::WriteOutput(redirection_context::RedirectionChannel::Stdout)
        );
    }

    #[test]
    fn parse_redirection_with_spaces() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello   >   {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
    }

    #[test]
    fn parse_redirection_with_quoted_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("file with spaces.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello > \"{}\"", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
    }

    #[test]
    fn parse_command_without_redirection() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello world");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello", "world"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_redirection_at_end() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("cat file1 file2 > {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["file1", "file2"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
    }

    #[test]
    fn parse_redirection_with_single_quoted_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output file.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo test > '{}'", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["test"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
    }

    #[test]
    fn parse_redirection_with_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello > {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
    }

    #[test]
    fn parse_quoted_command_with_redirection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("out.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("\"echo\" hello > {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);
        assert!(redirection.is_some());
    }

    #[test]
    fn parse_redirection_in_middle_is_not_detected() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo 'hello > world' test");

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello > world", "test"]);
        assert!(redirection.is_none());
    }

    #[test]
    fn parse_multiple_words_after_redirection_operator() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        // Should only take the first word/quoted string as filename
        let result = parser.parse(&format!("echo hello > {} extra", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert!(parsed.args().contains(&"hello".to_string()));
        assert!(redirection.is_some());
    }

    // ========================================================================
    // Append Redirection Tests (>>)
    // ========================================================================

    #[test]
    fn parse_simple_append_redirection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello >> {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        // Check append redirection was detected
        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert_eq!(
            redir.redirection_type,
            RedirectionType::AppendOutput(redirection_context::RedirectionChannel::Stdout)
        );
    }

    #[test]
    fn parse_append_vs_write_redirection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path_write = temp_dir.path().join("write.txt");
        let temp_path_append = temp_dir.path().join("append.txt");
        let temp_path_write_str = temp_path_write.to_str().unwrap();
        let temp_path_append_str = temp_path_append.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));

        // Parse write redirection
        let result_write = parser.parse(&format!("echo test > {}", temp_path_write_str));
        assert!(result_write.is_ok());
        let (_, redir_write) = result_write.unwrap();

        // Parse append redirection
        let result_append = parser.parse(&format!("echo test >> {}", temp_path_append_str));
        assert!(result_append.is_ok());
        let (_, redir_append) = result_append.unwrap();

        // Verify they are different types
        assert_ne!(
            redir_write.unwrap().redirection_type,
            redir_append.unwrap().redirection_type,
            "Write (>) and append (>>) should be different redirection types"
        );
    }

    #[test]
    fn parse_append_with_quoted_filename() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("file with spaces.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo test >> \"{}\"", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["test"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert!(redir.should_append_stdout());
    }

    #[test]
    fn parse_append_with_spaces_around_operator() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("echo hello   >>   {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "echo");
        assert_eq!(parsed.args(), &["hello"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert_eq!(
            redir.redirection_type,
            RedirectionType::AppendOutput(RedirectionChannel::Stdout)
        );
    }

    #[test]
    fn parse_append_at_end_of_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("output.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("cat file1 file2 >> {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["file1", "file2"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert!(redir.should_append_stdout());
    }

    // ========================================================================
    // Append Stderr Redirection Tests (2>>)
    // ========================================================================

    #[test]
    fn parse_simple_append_stderr_redirection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("errors.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("ls /tmp 2>> {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "ls");
        assert_eq!(parsed.args(), &["/tmp"]);

        // Check append stderr redirection was detected
        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert_eq!(
            redir.redirection_type,
            RedirectionType::AppendOutput(redirection_context::RedirectionChannel::Stderr)
        );
        assert!(redir.should_append_stderr());
    }

    #[test]
    fn parse_append_stderr_vs_append_stdout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path_stdout = temp_dir.path().join("stdout.txt");
        let temp_path_stderr = temp_dir.path().join("stderr.txt");
        let temp_path_stdout_str = temp_path_stdout.to_str().unwrap();
        let temp_path_stderr_str = temp_path_stderr.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));

        // Parse append stdout
        let result_stdout = parser.parse(&format!("echo test >> {}", temp_path_stdout_str));
        assert!(result_stdout.is_ok());
        let (_, redir_stdout) = result_stdout.unwrap();

        // Parse append stderr
        let result_stderr = parser.parse(&format!("echo test 2>> {}", temp_path_stderr_str));
        assert!(result_stderr.is_ok());
        let (_, redir_stderr) = result_stderr.unwrap();

        // Verify they redirect to different channels
        assert!(redir_stdout.as_ref().unwrap().should_append_stdout());
        assert!(redir_stderr.as_ref().unwrap().should_append_stderr());
        assert_ne!(
            redir_stdout.unwrap().redirection_type,
            redir_stderr.unwrap().redirection_type,
            "Append stdout (>>) and append stderr (2>>) should be different"
        );
    }

    #[test]
    fn parse_append_stderr_with_spaces() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("errors.txt");
        let temp_path_str = temp_path.to_str().unwrap();

        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse(&format!("cat file   2>>   {}", temp_path_str));

        assert!(result.is_ok());
        let (parsed, redirection) = result.unwrap();
        assert_eq!(parsed.command(), "cat");
        assert_eq!(parsed.args(), &["file"]);

        assert!(redirection.is_some());
        let redir = redirection.as_ref().unwrap();
        assert_eq!(redir.path, temp_path);
        assert!(redir.should_append_stderr());
    }

    // ========================================================================
    // Edge Case Tests - Redirection Errors
    // ========================================================================

    #[test]
    fn parse_redirection_without_filename_returns_error() {
        let parser = InputParser::new(Arc::new(FileManager));
        let result = parser.parse("echo hello >");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ShellError::Uncontroled(_)),
            "Expected Unknown error for missing filename after >"
        );
    }
}
