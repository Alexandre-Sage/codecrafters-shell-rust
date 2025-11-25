use std::{io, os::fd::AsRawFd};

use crate::exceptions::commands::ShellError;

pub struct RawMode {
    original_termios: Option<libc::termios>,
}

impl RawMode {
    pub fn enable() -> Result<Self, ShellError> {
        let stdin_fd = io::stdin().as_raw_fd();

        if unsafe { libc::isatty(stdin_fd) } == 0 {
            return Ok(RawMode {
                original_termios: None,
            });
        }

        let mut termios: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(stdin_fd, &mut termios) } != 0 {
            return Err(ShellError::Uncontroled(
                io::Error::last_os_error().to_string(),
            ));
        }

        let original_termios = termios;

        unsafe {
            libc::cfmakeraw(&mut termios);
        }

        // Apply new settings
        if unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios) } != 0 {
            return Err(ShellError::Uncontroled(
                io::Error::last_os_error().to_string(),
            ));
        }

        Ok(RawMode {
            original_termios: Some(original_termios),
        })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Only restore if we actually enabled raw mode
        if let Some(original_termios) = self.original_termios {
            let stdin_fd = io::stdin().as_raw_fd();
            unsafe {
                libc::tcsetattr(stdin_fd, libc::TCSANOW, &original_termios);
            }
        }
    }
}
