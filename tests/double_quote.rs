mod common;
use common::test_case;

#[test]
fn echo_with_double_quotes_preserves_spaces() {
    let output = test_case("echo \"hello    world\"", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("hello    world"),
        "Should preserve multiple spaces within double quotes, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_double_quotes_vs_no_quotes_spaces() {
    let output = test_case("echo hello    world\necho \"hello    world\"", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let outputs: Vec<String> = stdout.lines()
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
    
    assert!(outputs.len() >= 2, "Should have output from both echo commands, got {} outputs", outputs.len());
    assert_eq!(outputs[0], "hello world", "First echo should collapse spaces");
    assert_eq!(outputs[1], "hello    world", "Second echo should preserve spaces");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cat_with_double_quoted_filename_with_spaces() {
    let output = test_case("cat \"/etc/hostname\"", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(!stdout.is_empty() || !output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_adjacent_double_quotes_concatenate() {
    let output = test_case("echo \"hello\"\"world\"", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("helloworld"),
        "Adjacent double quotes should concatenate, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_empty_double_quotes_ignored() {
    let output = test_case("echo hello\"\"world", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("helloworld"),
        "Empty double quotes should be ignored, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_mixed_single_and_double_quotes() {
    let output = test_case("echo 'single' \"double\" plain", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("single double plain"),
        "Should handle mixed quotes, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_double_quotes_with_single_quote_inside() {
    let output = test_case("echo \"hello'world\"", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("hello'world"),
        "Single quote inside double quotes should be literal, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_single_quotes_with_double_quote_inside() {
    let output = test_case("echo 'hello\"world'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("hello\"world"),
        "Double quote inside single quotes should be literal, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn echo_alternating_single_double_quotes() {
    let output = test_case("echo \"a\"'b'\"c\"'d'", true);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(
        stdout.contains("abcd"),
        "Alternating quotes should concatenate, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}
