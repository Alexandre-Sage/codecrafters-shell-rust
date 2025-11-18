mod common;
use common::test_case;

#[test]
fn type_echo_builtin_command() {
    let output = test_case("type echo", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("echo is a shell builtin"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_exit_builtin_command() {
    let output = test_case("type exit", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("exit is a shell builtin"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_type_itself() {
    let output = test_case("type type", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("type is a shell builtin"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_unknown_command() {
    let output = test_case("type xyz", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("xyz: not found"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_no_arguments() {
    let output = test_case("type", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("No args"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_too_many_arguments() {
    let output = test_case("type echo exit", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Too many arguments"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_external_command_success() {
    let output = test_case("type grep", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("grep is /usr/bin/grep"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn type_external_command_not_found() {
    let output = test_case("type nonexistentcommand123", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("nonexistentcommand123: not found"));
    assert_eq!(output.status.code(), Some(0));
}
