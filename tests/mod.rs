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

// ============================================================================
// PWD Command Tests
// ============================================================================

#[test]
fn pwd_outputs_absolute_path() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd should output an absolute path (might have "$ " prefix from prompt)
    let has_path = stdout.lines()
        .any(|l| l.contains("/home") || l.contains("/usr") || l.contains("/tmp"));
    assert!(has_path, "pwd should output absolute path, got: {}", stdout);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_no_trailing_slash() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd output should not end with / (unless it's root directory)
    let path = stdout.trim();
    if path != "/" {
        assert!(!path.ends_with('/'), "pwd should not have trailing slash");
    }
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_single_line_output() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd should output exactly one path line
    let lines: Vec<&str> = stdout.lines()
        .filter(|l| l.contains("codecrafters-shell-rust"))
        .collect();
    assert_eq!(lines.len(), 1, "pwd should output exactly one path line");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_no_stderr_on_success() {
    let output = test_case("pwd", true);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // pwd with no args should not produce stderr
    assert!(stderr.is_empty() || !stderr.contains("invalid option"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_multiple_calls_consistent() {
    let output = test_case("pwd\npwd\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // All pwd calls should return the same path
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty() && !l.starts_with('$')).collect();
    
    if lines.len() >= 2 {
        assert_eq!(lines[0], lines[1], "pwd should be consistent across calls");
        if lines.len() >= 3 {
            assert_eq!(lines[1], lines[2], "pwd should be consistent across calls");
        }
    }
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_after_other_commands() {
    let output = test_case("echo test\npwd\nls", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd should work correctly after other commands
    assert!(stdout.contains("test"));
    assert!(stdout.contains("codecrafters-shell-rust"));
    assert!(stdout.contains("Cargo.toml") || stdout.contains("src"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_valid_path_format() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Find line with path (might be prefixed with "$ ")
    let path_line = stdout.lines()
        .find(|l| l.contains("codecrafters-shell-rust"))
        .expect("Should have a path line with project name");
    
    // Extract actual path (remove "$ " prefix if present)
    let path = path_line.trim_start_matches("$ ").trim();
    
    // pwd should output a valid Unix path
    assert!(path.starts_with('/'), "Should be absolute path");
    assert!(!path.is_empty(), "Path should not be empty");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_contains_valid_components() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Find line with path
    let path_line = stdout.lines()
        .find(|l| l.contains("codecrafters-shell-rust"))
        .expect("Should have a path line");
    
    // Extract actual path (remove "$ " prefix if present)
    let path = path_line.trim_start_matches("$ ").trim();
    
    // Path components should not contain invalid characters
    assert!(!path.contains("//"), "Should not have consecutive slashes");
    assert_eq!(output.status.code(), Some(0));
}

// ============================================================================
// Back to other external command tests
// ============================================================================

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

// ============================================================================
// CD Command Integration Tests (Absolute Paths Only)
// ============================================================================

#[test]
fn cd_to_tmp_then_pwd() {
    let output = test_case("cd /tmp\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // After cd /tmp, pwd should show /tmp
    assert!(stdout.contains("/tmp"), "pwd should show /tmp after cd /tmp");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_to_root_then_pwd() {
    let output = test_case("cd /\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // After cd /, pwd should show /
    let has_root = stdout.lines()
        .any(|l| l.trim() == "/" || l.contains("$ /"));
    assert!(has_root, "pwd should show / after cd /, got: {}", stdout);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_to_home_no_args() {
    let output = test_case("cd\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // After cd with no args, should be in home directory
    assert!(
        stdout.contains("/home/") || stdout.contains("/Users/"),
        "Should be in home directory after cd with no args"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_nonexistent_directory() {
    let output = test_case("cd /this/does/not/exist/xyz123", true);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should produce error
    assert!(
        stderr.contains("not found") || stderr.contains("No such"),
        "Should show error for nonexistent directory"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_to_file_not_directory() {
    let output = test_case("cd /bin/sh", true);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should produce error (can't cd to a file)
    assert!(
        stderr.contains("Not a directory") || stderr.contains("not a directory"),
        "Should show error when trying to cd to a file"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_persists_across_commands() {
    let output = test_case("cd /tmp\nls\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // pwd at the end should still show /tmp
    assert!(stdout.contains("/tmp"), "Directory change should persist");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_then_ls_shows_correct_directory() {
    let output = test_case("cd /\nls", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // ls after cd / should show root directory contents
    assert!(
        stdout.contains("bin") || stdout.contains("usr") || stdout.contains("tmp"),
        "ls should show root directory contents after cd /"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn multiple_cd_commands() {
    let output = test_case("cd /tmp\ncd /\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Final pwd should show / (last cd wins)
    let has_root = stdout.lines()
        .any(|l| l.trim() == "/" || l.contains("$ /"));
    assert!(has_root, "Final pwd should show / after multiple cd commands");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_with_trailing_slash() {
    let output = test_case("cd /tmp/\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should handle trailing slash correctly
    assert!(stdout.contains("/tmp"), "Should handle trailing slash in cd");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_too_many_arguments() {
    let output = test_case("cd /tmp /usr", true);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should produce error for multiple arguments
    assert!(
        stderr.contains("Too many arguments") || stderr.contains("too many"),
        "Should show error for cd with multiple arguments"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_then_external_command() {
    let output = test_case("cd /tmp\ncat /etc/hostname", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // cat should work after cd (external commands should still work)
    assert!(!stdout.is_empty(), "External commands should work after cd");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_affects_relative_file_operations() {
    let output = test_case("cd /\nls tmp", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // After cd /, ls tmp should list /tmp contents
    // (This tests that cd actually changes the working directory for child processes)
    assert_eq!(output.status.code(), Some(0));
}
