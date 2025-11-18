use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn test_case(command: &str, should_exit: bool) -> std::process::Output {
    let mut child_proc = Command::new("cargo")
        .args(["run", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    {
        let stdin = child_proc.stdin.as_mut().expect("Failed to open stdin");
        let command = if should_exit {
            command.to_string() + "\nexit\n"
        } else {
            command.to_owned()
        };
        stdin
            .write_all(command.as_bytes())
            .expect("failed to write to stdin");
    }

    child_proc
        .wait_with_output()
        .expect("Failed to read output")
}
