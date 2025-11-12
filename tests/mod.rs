use std::{
    io::Write,
    process::{Command, Stdio},
};

fn test_case(command: &str, should_exit: bool) -> std::process::Output {
    let mut child_proc = Command::new("cargo")
        .args(&["run", "--quiet"])
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

#[test]
fn exit_command_no_args() {
    let output = test_case("exit", false);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn exit_command_with_code() {
    let output = test_case("exit 42", false);

    assert_eq!(output.status.code(), Some(42));
}

#[test]
fn invalid_command_shows_error() {
    let output = test_case("invalidcmd", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("command not found"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn exit_with_invalid_argument() {
    let output = test_case("exit abc", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Invalid arg type"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn exit_with_too_many_arguments() {
    let output = test_case("exit 1 2", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Too many arguments"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_prints_hello_world() {
    let output = test_case("echo hello world", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("hello world"));
    assert_eq!(output.status.code(), Some(0));
}
