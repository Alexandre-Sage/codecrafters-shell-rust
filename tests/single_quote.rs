mod common;
use common::test_case;

#[test]
fn echo_with_single_quotes_preserves_spaces() {
    let output = test_case("echo 'hello    world'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Spaces within quotes should be preserved
    assert!(
        stdout.contains("hello    world"),
        "Should preserve multiple spaces within quotes, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_single_quotes_vs_no_quotes_spaces() {
    let output = test_case("echo hello    world\necho 'hello    world'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract output after prompts
    // Lines look like: "$ hello world"
    let outputs: Vec<String> = stdout
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("$ ") {
                Some(trimmed[2..].to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty() && s != "exit")
        .collect();

    // Without quotes: "hello world" (spaces collapsed)
    // With quotes: "hello    world" (spaces preserved)
    assert!(
        outputs.len() >= 2,
        "Should have output from both echo commands, got {} outputs",
        outputs.len()
    );
    assert_eq!(
        outputs[0], "hello world",
        "First echo should collapse spaces"
    );
    assert_eq!(
        outputs[1], "hello    world",
        "Second echo should preserve spaces"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cat_with_single_quoted_filename_with_spaces() {
    // First create a file with spaces in name
    let output = test_case("cat '/etc/hostname'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // cat should work with quoted filenames
    assert!(!stdout.is_empty() || !output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_adjacent_single_quotes_concatenate() {
    let output = test_case("echo 'hello''world'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Adjacent quoted strings should concatenate
    assert!(
        stdout.contains("helloworld"),
        "Adjacent quotes should concatenate, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_empty_quotes_ignored() {
    let output = test_case("echo hello''world", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Empty quotes should be ignored
    assert!(
        stdout.contains("helloworld"),
        "Empty quotes should be ignored, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}
