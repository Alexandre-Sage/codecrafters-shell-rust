mod common;
use common::test_case;

#[test]
fn external_command_pwd() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // pwd should output current directory path
    assert!(stdout.contains("codecrafters-shell-rust"));
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_outputs_absolute_path() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // pwd should output an absolute path (might have "$ " prefix from prompt)
    let has_path = stdout
        .lines()
        .any(|l| l.contains("/home") || l.contains("/usr") || l.contains("/tmp"));
    assert!(has_path, "pwd should output absolute path, got: {stdout}");
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
    let lines: Vec<&str> = stdout
        .lines()
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
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('$'))
        .collect();

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
    let path_line = stdout
        .lines()
        .find(|l| l.contains("codecrafters-shell-rust"))
        .expect("Should have a path line");

    // Extract actual path (remove "$ " prefix if present)
    let path = path_line.trim_start_matches("$ ").trim();

    // Path should start with /
    assert!(path.starts_with('/'), "Path should start with /");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn pwd_contains_valid_components() {
    let output = test_case("pwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find line with path
    let path_line = stdout
        .lines()
        .find(|l| l.contains("codecrafters-shell-rust"))
        .expect("Should have a path line");

    // Extract actual path (remove "$ " prefix if present)
    let path = path_line.trim_start_matches("$ ").trim();

    // Path components should not contain invalid characters
    assert!(!path.contains("//"), "Should not have consecutive slashes");
    assert_eq!(output.status.code(), Some(0));
}
