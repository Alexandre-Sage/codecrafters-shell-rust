mod common;

use common::test_case;
use std::fs;

// ========================================================================
// Integration Tests - Output Redirection
// ========================================================================

#[test]
fn echo_redirect_to_file() {
    let output_file = "/tmp/test_echo_redirect.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    let output = test_case("echo hello world > /tmp/test_echo_redirect.txt", true);
    
    // Check that file was created
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file should be created"
    );
    
    // Check file contents
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(
        contents.trim(),
        "hello world",
        "File should contain the echo output"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn echo_redirect_with_quoted_filename() {
    let output_file = "/tmp/test file with spaces.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    let output = test_case("echo test > \"/tmp/test file with spaces.txt\"", true);
    
    // Check that file was created
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file with spaces should be created"
    );
    
    // Check file contents
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(contents.trim(), "test");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn redirect_overwrites_existing_file() {
    let output_file = "/tmp/test_overwrite.txt";
    
    // Create file with initial content
    fs::write(output_file, "old content").unwrap();
    
    let output = test_case("echo new content > /tmp/test_overwrite.txt", true);
    
    // Check that file was overwritten
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(
        contents.trim(),
        "new content",
        "File should be overwritten with new content"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn external_command_redirect() {
    let output_file = "/tmp/test_ls_redirect.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    let output = test_case("ls /tmp > /tmp/test_ls_redirect.txt", true);
    
    // Check that file was created
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file should be created for external command"
    );
    
    // Check that file has content (ls should output something)
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(
        !contents.is_empty(),
        "ls output should not be empty"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn redirect_with_special_characters_in_output() {
    let output_file = "/tmp/test_special_chars.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    let output = test_case("echo 'hello world' > /tmp/test_special_chars.txt", true);
    
    // Check that file was created
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file should be created"
    );
    
    // Check file contents preserve the single quotes' content
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(contents.trim(), "hello world");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}
