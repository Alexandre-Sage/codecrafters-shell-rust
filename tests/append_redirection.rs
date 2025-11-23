mod common;

use common::test_case;
use std::fs;

// ========================================================================
// Integration Tests - Append Redirection (>>)
// ========================================================================

#[test]
fn append_to_new_file() {
    let output_file = "/tmp/test_append_new.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    test_case("echo first line >> /tmp/test_append_new.txt", true);
    
    // Check that file was created
    assert!(
        fs::metadata(output_file).is_ok(),
        "Output file should be created with append operator"
    );
    
    // Check file contents
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(
        contents.trim(),
        "first line",
        "File should contain the first line"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_to_existing_file() {
    let output_file = "/tmp/test_append_existing.txt";
    
    // Create file with initial content
    fs::write(output_file, "line 1\n").unwrap();
    
    test_case("echo line 2 >> /tmp/test_append_existing.txt", true);
    
    // Check that content was appended, not overwritten
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(
        contents.contains("line 1"),
        "Original content should be preserved"
    );
    assert!(
        contents.contains("line 2"),
        "New content should be appended"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn multiple_appends_to_same_file() {
    let output_file = "/tmp/test_multiple_appends.txt";
    
    // Clean up any existing file
    fs::remove_file(output_file).ok();
    
    test_case("echo line 1 >> /tmp/test_multiple_appends.txt", true);
    test_case("echo line 2 >> /tmp/test_multiple_appends.txt", true);
    test_case("echo line 3 >> /tmp/test_multiple_appends.txt", true);
    
    // Check that all lines are present in order
    let contents = fs::read_to_string(output_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    
    assert_eq!(lines.len(), 3, "Should have 3 lines");
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[1], "line 2");
    assert_eq!(lines[2], "line 3");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_vs_overwrite_behavior() {
    let output_file = "/tmp/test_append_vs_overwrite.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    // First write with >
    test_case("echo first >> /tmp/test_append_vs_overwrite.txt", true);
    test_case("echo second > /tmp/test_append_vs_overwrite.txt", true);
    
    // Check that > overwrote the file
    let contents = fs::read_to_string(output_file).unwrap();
    assert_eq!(
        contents.trim(),
        "second",
        "Overwrite (>) should replace content"
    );
    assert!(
        !contents.contains("first"),
        "Previous content should be gone after overwrite"
    );
    
    // Now append again
    test_case("echo third >> /tmp/test_append_vs_overwrite.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(contents.contains("second"), "Second line should remain");
    assert!(contents.contains("third"), "Third line should be appended");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_empty_output() {
    let output_file = "/tmp/test_append_empty.txt";
    
    // Create file with content
    fs::write(output_file, "existing content\n").unwrap();
    
    // Append empty echo
    test_case("echo >> /tmp/test_append_empty.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(
        contents.contains("existing content"),
        "Original content should be preserved"
    );
    // Empty echo adds just a newline
    assert!(contents.len() > "existing content\n".len());
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_with_quoted_filename() {
    let output_file = "/tmp/test append file.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    test_case("echo first >> \"/tmp/test append file.txt\"", true);
    test_case("echo second >> \"/tmp/test append file.txt\"", true);
    
    // Check both lines are present
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(contents.contains("first"));
    assert!(contents.contains("second"));
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_external_command_output() {
    let output_file = "/tmp/test_append_external.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    // First append
    test_case("echo '=== PWD ===' >> /tmp/test_append_external.txt", true);
    // Append external command output
    test_case("pwd >> /tmp/test_append_external.txt", true);
    // Another append
    test_case("echo '=== END ===' >> /tmp/test_append_external.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(contents.contains("=== PWD ==="));
    assert!(contents.contains("=== END ==="));
    // pwd should output something (current directory path)
    let lines: Vec<&str> = contents.lines().collect();
    assert!(lines.len() >= 3, "Should have at least 3 lines");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_preserves_file_size_growth() {
    let output_file = "/tmp/test_append_growth.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    test_case("echo a >> /tmp/test_append_growth.txt", true);
    let size1 = fs::metadata(output_file).unwrap().len();
    
    test_case("echo bb >> /tmp/test_append_growth.txt", true);
    let size2 = fs::metadata(output_file).unwrap().len();
    
    test_case("echo ccc >> /tmp/test_append_growth.txt", true);
    let size3 = fs::metadata(output_file).unwrap().len();
    
    // Each append should increase file size
    assert!(size2 > size1, "File should grow after second append");
    assert!(size3 > size2, "File should grow after third append");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

// ========================================================================
// Integration Tests - Append Stderr Redirection (2>>)
// ========================================================================

#[test]
fn append_stderr_to_new_file() {
    let output_file = "/tmp/test_append_stderr_new.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    // ls on non-existent directory produces stderr
    test_case("ls /nonexistent_dir_xyz 2>> /tmp/test_append_stderr_new.txt", true);
    
    // Check that file was created with stderr
    assert!(
        fs::metadata(output_file).is_ok(),
        "Stderr file should be created with 2>>"
    );
    
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(
        !contents.is_empty(),
        "Stderr should be captured in file"
    );
    assert!(
        contents.contains("nonexistent_dir_xyz") || contents.contains("No such file"),
        "Error message should mention the missing directory"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_stderr_to_existing_file() {
    let output_file = "/tmp/test_append_stderr_existing.txt";
    
    // Create file with initial content
    fs::write(output_file, "=== Previous Error ===\n").unwrap();
    
    test_case("ls /another_nonexistent 2>> /tmp/test_append_stderr_existing.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    assert!(
        contents.contains("=== Previous Error ==="),
        "Original content should be preserved"
    );
    assert!(
        contents.contains("another_nonexistent") || contents.contains("No such file"),
        "New stderr should be appended"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_stderr_multiple_times() {
    let output_file = "/tmp/test_append_stderr_multiple.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    test_case("ls /err1 2>> /tmp/test_append_stderr_multiple.txt", true);
    test_case("ls /err2 2>> /tmp/test_append_stderr_multiple.txt", true);
    test_case("ls /err3 2>> /tmp/test_append_stderr_multiple.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    
    // All three errors should be present
    let has_all = (contents.contains("err1") || contents.matches("No such file").count() >= 1)
        && (contents.contains("err2") || contents.matches("No such file").count() >= 2)
        && (contents.contains("err3") || contents.matches("No such file").count() >= 3);
    
    assert!(has_all, "All three stderr outputs should be appended");
    
    // Cleanup
    fs::remove_file(output_file).ok();
}

#[test]
fn append_stderr_vs_overwrite_stderr() {
    let output_file = "/tmp/test_append_vs_overwrite_stderr.txt";
    
    // Clean up
    fs::remove_file(output_file).ok();
    
    // First append
    test_case("ls /error1 2>> /tmp/test_append_vs_overwrite_stderr.txt", true);
    
    // Then overwrite with 2>
    test_case("ls /error2 2> /tmp/test_append_vs_overwrite_stderr.txt", true);
    
    let contents = fs::read_to_string(output_file).unwrap();
    
    // Should only have error2, not error1 (because 2> overwrites)
    assert!(
        !contents.contains("error1"),
        "First error should be overwritten"
    );
    assert!(
        contents.contains("error2") || contents.contains("No such file"),
        "Second error should be present"
    );
    
    // Cleanup
    fs::remove_file(output_file).ok();
}
