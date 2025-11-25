use std::io::{self, Read, Write};

use crate::{
    exceptions::commands::ShellError,
    shell::{
        completion::{self, builtins::BuiltinsCompletion, CompletionComponent},
        input::commons::{
            BACK_SPACE, BELL_CHAR, CARRIAGE, CRLF, CTRL_C, CTRL_H, LINEBREAK, TABULATION,
        },
        raw_mode::RawMode,
    },
};

pub(crate) struct InputHandler {
    completion: BuiltinsCompletion,
}

impl InputHandler {
    pub(crate) fn new(completion: BuiltinsCompletion) -> Self {
        Self { completion }
    }

    pub(crate) fn handle(&self) -> Result<Option<String>, ShellError> {
        let _raw_mode = RawMode::enable()?;

        let mut buffer = String::new();
        let mut tmp_buffer = [0u8; 1];
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();

        loop {
            stdin
                .read_exact(&mut tmp_buffer[..])
                .map_err(|e| ShellError::Uncontroled(e.to_string()))?;

            match tmp_buffer[0] {
                TABULATION => {
                    let completion = self.completion.execute(&buffer);
                    match completion {
                        Some(completion_item) => {
                            let completion_item = format!("{completion_item} ");
                            buffer.push_str(&completion_item);
                            self.write_output(&mut stdout, completion_item.as_bytes())?;
                        }
                        None => {
                            self.write_output(&mut stdout, BELL_CHAR.as_bytes())?;
                        }
                    }
                }
                CARRIAGE | LINEBREAK => {
                    self.write_output(&mut stdout, CRLF.as_bytes())?;
                    break;
                }
                BACK_SPACE | CTRL_H => {
                    if !buffer.is_empty() {
                        buffer.pop();
                        self.write_output(&mut stdout, b"\x08 \x08")?;
                    }
                }
                CTRL_C => {
                    self.write_output(&mut stdout, b"^C\r\n")?;
                    return Ok(None);
                }
                c if c >= 32 && c < 127 => {
                    buffer.push(c as char);
                    self.write_output(&mut stdout, &[c])?;
                }
                _ => {}
            }
        }

        Ok(Some(buffer))
    }

    fn write_output(&self, writer: &mut impl Write, buffer: &[u8]) -> Result<(), ShellError> {
        writer
            .write_all(buffer)
            .and_then(|_| writer.flush())
            .map_err(|err| ShellError::Uncontroled(err.to_string()))
    }

    // Internal method for processing input - testable without raw mode
    fn process_byte(&self, byte: u8, buffer: &mut String) -> ProcessResult {
        match byte {
            TABULATION => {
                if let Some(completion_suffix) = self.completion.complete(buffer) {
                    buffer.push_str(&completion_suffix);
                    ProcessResult::Completion(completion_suffix)
                } else {
                    ProcessResult::NoCompletion
                }
            }
            CARRIAGE | LINEBREAK => ProcessResult::Submit,
            BACK_SPACE | CTRL_H => {
                if !buffer.is_empty() {
                    buffer.pop();
                    ProcessResult::Backspace
                } else {
                    ProcessResult::Ignore
                }
            }
            CTRL_C => ProcessResult::Interrupted,
            c if c >= 32 && c < 127 => {
                buffer.push(c as char);
                ProcessResult::Echo(c as char)
            }
            _ => ProcessResult::Ignore,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ProcessResult {
    Echo(char),
    Backspace,
    Completion(String),
    NoCompletion,
    Submit,
    Interrupted,
    Ignore,
}

#[cfg(test)]
mod tests {
    use crate::shell::path::PathDirs;

    use super::*;

    fn setup() -> InputHandler {
        let path = PathDirs::from_env();
        let completion = BuiltinsCompletion::new(path.into());
        InputHandler::new(completion)
    }

    #[test]
    fn process_printable_characters() {
        let handler = setup();
        let mut buffer = String::new();

        assert_eq!(
            handler.process_byte(b'h', &mut buffer),
            ProcessResult::Echo('h')
        );
        assert_eq!(buffer, "h");

        assert_eq!(
            handler.process_byte(b'i', &mut buffer),
            ProcessResult::Echo('i')
        );
        assert_eq!(buffer, "hi");
    }

    #[test]
    fn process_backspace_removes_character() {
        let handler = setup();
        let mut buffer = String::from("hello");

        assert_eq!(
            handler.process_byte(BACK_SPACE, &mut buffer),
            ProcessResult::Backspace
        );
        assert_eq!(buffer, "hell");

        assert_eq!(
            handler.process_byte(CTRL_H, &mut buffer),
            ProcessResult::Backspace
        );
        assert_eq!(buffer, "hel");
    }

    #[test]
    fn process_backspace_on_empty_buffer_does_nothing() {
        let handler = setup();
        let mut buffer = String::new();

        assert_eq!(
            handler.process_byte(BACK_SPACE, &mut buffer),
            ProcessResult::Ignore
        );
        assert_eq!(buffer, "");
    }

    #[test]
    fn process_tab_completes_unambiguous_command() {
        let handler = setup();
        let mut buffer = String::from("ec");

        assert_eq!(
            handler.process_byte(TABULATION, &mut buffer),
            ProcessResult::Completion("ho".to_string())
        );
        assert_eq!(buffer, "echo");
    }

    #[test]
    fn process_tab_with_ambiguous_prefix_does_nothing() {
        let handler = setup();
        let mut buffer = String::from("e");

        assert_eq!(
            handler.process_byte(TABULATION, &mut buffer),
            ProcessResult::NoCompletion
        );
        assert_eq!(buffer, "e");
    }

    #[test]
    fn process_tab_with_no_match_does_nothing() {
        let handler = setup();
        let mut buffer = String::from("xyz");

        assert_eq!(
            handler.process_byte(TABULATION, &mut buffer),
            ProcessResult::NoCompletion
        );
        assert_eq!(buffer, "xyz");
    }

    #[test]
    fn process_enter_submits_input() {
        let handler = setup();
        let mut buffer = String::from("echo test");

        assert_eq!(
            handler.process_byte(LINEBREAK, &mut buffer),
            ProcessResult::Submit
        );
        assert_eq!(buffer, "echo test");

        let mut buffer2 = String::from("pwd");
        assert_eq!(
            handler.process_byte(CARRIAGE, &mut buffer2),
            ProcessResult::Submit
        );
        assert_eq!(buffer2, "pwd");
    }

    #[test]
    fn process_ctrl_c_interrupts() {
        let handler = setup();
        let mut buffer = String::from("some input");

        assert_eq!(
            handler.process_byte(CTRL_C, &mut buffer),
            ProcessResult::Interrupted
        );
        assert_eq!(buffer, "some input"); // Buffer unchanged
    }

    #[test]
    fn process_control_characters_are_ignored() {
        let handler = setup();
        let mut buffer = String::new();

        // Control characters < 32 (except handled ones)
        assert_eq!(handler.process_byte(0, &mut buffer), ProcessResult::Ignore);
        assert_eq!(handler.process_byte(1, &mut buffer), ProcessResult::Ignore);
        assert_eq!(handler.process_byte(27, &mut buffer), ProcessResult::Ignore); // ESC

        // DEL and beyond
        assert_eq!(
            handler.process_byte(128, &mut buffer),
            ProcessResult::Ignore
        );
        assert_eq!(
            handler.process_byte(255, &mut buffer),
            ProcessResult::Ignore
        );

        assert_eq!(buffer, "");
    }

    #[test]
    fn process_full_command_input_sequence() {
        let handler = setup();
        let mut buffer = String::new();

        // Type "ec"
        handler.process_byte(b'e', &mut buffer);
        handler.process_byte(b'c', &mut buffer);
        assert_eq!(buffer, "ec");

        // Tab to complete to "echo"
        let result = handler.process_byte(TABULATION, &mut buffer);
        assert_eq!(result, ProcessResult::Completion("ho".to_string()));
        assert_eq!(buffer, "echo");

        // Add space and argument
        handler.process_byte(b' ', &mut buffer);
        handler.process_byte(b'h', &mut buffer);
        handler.process_byte(b'i', &mut buffer);
        assert_eq!(buffer, "echo hi");

        // Backspace once
        handler.process_byte(BACK_SPACE, &mut buffer);
        assert_eq!(buffer, "echo h");

        // Type "ello"
        handler.process_byte(b'e', &mut buffer);
        handler.process_byte(b'l', &mut buffer);
        handler.process_byte(b'l', &mut buffer);
        handler.process_byte(b'o', &mut buffer);
        assert_eq!(buffer, "echo hello");

        // Submit
        let result = handler.process_byte(LINEBREAK, &mut buffer);
        assert_eq!(result, ProcessResult::Submit);
        assert_eq!(buffer, "echo hello");
    }
}
