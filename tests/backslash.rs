mod common;

use common::test_case;

// ========================================================================
// Happy Path Tests - Backslash Escaping
// ========================================================================

#[test]
fn echo_backslash_escapes_space() {
    let output = test_case("echo hello\\ world", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"hello world".to_string()),
        "Expected 'hello world' in output, got: {lines:?}"
    );
}

#[test]
fn echo_multiple_escaped_spaces() {
    let output = test_case("echo world\\ \\ \\ \\ \\ \\ script", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"world      script".to_string()),
        "Expected 'world      script' (6 spaces) in output, got: {lines:?}"
    );
}

#[test]
fn echo_backslash_inside_double_quotes_preserved() {
    let output = test_case("echo \"before\\   after\"", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"before\\   after".to_string()),
        "Expected 'before\\   after' (backslash preserved) in output, got: {lines:?}"
    );
}

#[test]
fn echo_double_backslash_produces_single() {
    let output = test_case("echo hello\\\\world", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"hello\\world".to_string()),
        "Expected 'hello\\world' (single backslash) in output, got: {lines:?}"
    );
}

#[test]
fn echo_backslash_escapes_special_chars() {
    let output = test_case("echo \\$HOME \\* \\?", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"$HOME * ?".to_string()),
        "Expected '$HOME * ?' in output, got: {lines:?}"
    );
}

// ========================================================================
// File Name Tests with Backslashes
// ========================================================================

#[test]
fn cat_file_with_backslash_in_name() {
    // Create test file with space in name
    std::fs::write("/tmp/test_file name", "test content").unwrap();

    let output = test_case("cat /tmp/test_file\\ name", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test content"),
        "Expected 'test content' in output, got: {stdout}"
    );

    // Cleanup
    std::fs::remove_file("/tmp/test_file name").ok();
}

#[test]
fn cat_files_with_backslashes_inside_double_quotes() {
    // Create test files
    // Inside double quotes, backslashes are literal, so we need files with actual backslashes
    std::fs::write("/tmp/file\\\\name", "content1").unwrap();
    std::fs::write("/tmp/file\\ name", "content2").unwrap();

    let output = test_case("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\"", true);

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Inside double quotes, backslashes are preserved literally
    // So "/tmp/file\\name" looks for a file literally named with double-backslash
    // And "/tmp/file\ name" looks for a file with backslash-space
    assert!(
        stdout.contains("content1") && stdout.contains("content2"),
        "Expected both 'content1' and 'content2' in output, got: {stdout}"
    );

    // Cleanup
    std::fs::remove_file("/tmp/file\\\\name").ok();
    std::fs::remove_file("/tmp/file\\ name").ok();
}

// ========================================================================
// Edge Case Tests
// ========================================================================

#[test]
fn echo_backslash_not_escape_inside_single_quotes() {
    let output = test_case("echo 'hello\\ world'", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    // Inside single quotes, backslash is literal
    assert!(
        lines.contains(&"hello\\ world".to_string()),
        "Expected 'hello\\ world' (backslash literal) in output, got: {lines:?}"
    );
}

#[test]
fn echo_mixed_quotes_and_backslashes() {
    let output = test_case("echo \"quoted\"\\ unquoted", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"quoted unquoted".to_string()),
        "Expected 'quoted unquoted' in output, got: {lines:?}"
    );
}

#[test]
fn echo_escaped_quotes() {
    let output = test_case("echo \\\"hello\\\" world", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"\"hello\" world".to_string()),
        "Expected '\"hello\" world' in output, got: {lines:?}"
    );
}

#[test]
fn echo_backslash_with_regular_chars() {
    let output = test_case("echo \\a\\b\\c", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"abc".to_string()),
        "Expected 'abc' in output, got: {lines:?}"
    );
}

#[test]
fn echo_mixed_escaped_and_unescaped_spaces() {
    let output = test_case("echo hello\\ world foo", true);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = stdout
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                trimmed[2..].to_string()
            } else if trimmed.starts_with('$') {
                String::new()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|l| !l.is_empty() && !l.starts_with("warning:") && !l.starts_with("-->"))
        .collect();

    assert!(
        lines.contains(&"hello world foo".to_string()),
        "Expected 'hello world foo' in output, got: {lines:?}"
    );
}
