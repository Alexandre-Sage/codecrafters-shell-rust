use std::{
    io::Write,
    process::{Command, Stdio},
};

fn test_case(command: &str, should_exit: bool) -> std::process::Output {
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

// ============================================================================
// External Command Execution Tests
// ============================================================================

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
fn external_command_pwd() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd should output current directory path
    assert!(stdout.contains("codecrafters-shell-rust"));
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
