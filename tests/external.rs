mod common;
use common::test_case;

// Happy Path Tests

#[test]
fn external_command_ls_no_args() {
    let output = test_case("ls", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // ls should list files in current directory (at minimum Cargo.toml exists)
    assert!(stdout.contains("Cargo.toml") || stdout.contains("src"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_cat_with_file() {
    let output = test_case("cat Cargo.toml", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Cargo.toml should contain package info
    assert!(stdout.contains("codecrafters-shell") || stdout.contains("[package]"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_grep_simple() {
    let output = test_case("grep package Cargo.toml", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should find lines with "package"
    assert!(stdout.contains("package") || stdout.contains("name"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_multiple_sequential() {
    let output = test_case("ls\npwd\necho test", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // All three commands should execute
    assert!(stdout.contains("Cargo.toml") || stdout.contains("src"));
    assert!(stdout.contains("codecrafters-shell-rust"));
    assert!(stdout.contains("test"));
    assert_eq!(output.status.code(), Some(0));
}

// Error Cases

#[test]
fn external_command_not_found() {
    let output = test_case("nonexistentcommand123", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("command not found") || stderr.contains("not found"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_cat_nonexistent_file() {
    let output = test_case("cat nonexistentfile12345.txt", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // cat should report error on stderr
    assert!(stderr.contains("No such file") || stderr.contains("cannot open"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_grep_no_match() {
    let output = test_case("grep xyznotfound123 Cargo.toml", true);

    // grep with no match returns exit code 1, but shell should continue
    assert_eq!(output.status.code(), Some(0));
}

// Edge Cases

#[test]
fn external_command_empty_output() {
    let output = test_case("ls /nonexistent/path/12345", true);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have error message
    assert!(stderr.contains("No such file") || stderr.contains("cannot access"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_with_many_args() {
    let output = test_case("echo one two three four five six seven", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("one two three four five six seven"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_with_special_chars() {
    let output = test_case("echo hello-world_test.file", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("hello-world_test.file"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn builtin_takes_precedence_over_external() {
    // Even if "echo" exists as external command, builtin should execute
    let output = test_case("echo builtin test", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Builtin echo should work
    assert!(stdout.contains("builtin test"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_after_builtin() {
    // Test that external commands work after builtins
    let output = test_case("echo from builtin\nls", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("from builtin"));
    assert!(stdout.contains("Cargo.toml") || stdout.contains("src"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_true_succeeds() {
    // /usr/bin/true always exits with 0
    let output = test_case("true", true);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn external_command_false_fails() {
    // /usr/bin/false always exits with 1, but shell should continue
    let output = test_case("false", true);

    // Shell itself should exit with 0 (from our exit command)
    assert_eq!(output.status.code(), Some(0));
}
