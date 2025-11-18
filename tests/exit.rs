mod common;
use common::test_case;

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
fn invalid_command_shows_error() {
    let output = test_case("invalidcmd", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("command not found"));
    assert_eq!(output.status.code(), Some(0));
}
