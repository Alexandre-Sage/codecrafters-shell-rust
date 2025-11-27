use std::io::{self, Read, Stdin, StdinLock, StdoutLock, Write};

use crate::{
    exceptions::commands::ShellError,
    shell::{
        completion::{self, builtins::BuiltinsCompletion, CompletionComponent},
        input::commons::{
            ASCII_DEL, ASCII_SPACE, BACK_SPACE, BELL_CHAR, CARRIAGE, CRLF, CTRL_C, CTRL_H,
            LINEBREAK, TABULATION,
        },
        raw_mode::RawMode,
    },
};

// struct InputState<'a> {
//     buffer: String,
//     stdin: StdinLock<'a>,
//     stdout: StdoutLock<'a>,
//     tab_pressed_once: bool,
// }

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

        let mut is_tab_pressed = false;

        loop {
            stdin
                .read_exact(&mut tmp_buffer[..])
                .map_err(|e| ShellError::Uncontroled(e.to_string()))?;

            match tmp_buffer[0] {
                TABULATION => {
                    let completion = self.completion.execute(&buffer, is_tab_pressed);
                    match completion {
                        Some(completion_item) => {
                            if is_tab_pressed {
                                self.write_output(
                                    &mut stdout,
                                    format!("{CRLF}{completion_item}").as_bytes(),
                                )?;
                                return Ok(None);
                            }

                            let completion_item = format!("{completion_item} ");
                            buffer.push_str(&completion_item);
                            self.write_output(&mut stdout, completion_item.as_bytes())?;
                        }
                        None => {
                            self.write_output(&mut stdout, BELL_CHAR.as_bytes())?;
                            is_tab_pressed = true;
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
                    is_tab_pressed = false;
                }
                CTRL_C => {
                    self.write_output(&mut stdout, b"^C\r\n")?;
                    return Ok(None);
                }
                c if c >= ASCII_SPACE && c < ASCII_DEL => {
                    buffer.push(c as char);
                    self.write_output(&mut stdout, &[c])?;
                    is_tab_pressed = false;
                }
                _ => {
                    is_tab_pressed = false;
                }
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
}
