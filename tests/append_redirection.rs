mod common;

use common::test_case;
use std::fs;

// ========================================================================
// Integration Tests - Append Redirection (>>)
// ========================================================================

#[test]
fn append_to_new_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_new.txt");
    
    test_case(&format!("echo first line >> {}", output_file.display()), true);
    
    // Check that file was created
    assert!(
        output_file.exists(),
        "Output file should be created with append operator"
    );
    
    // Check file contents
    let contents = fs::read_to_string(&output_file).unwrap();
    assert_eq!(
        contents.trim(),
        "first line",
        "File should contain the first line"
    );
}

#[test]
fn append_to_existing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_existing.txt");
    
    // Create file with initial content
    fs::write(&output_file, "line 1\n").unwrap();
    
    test_case(&format!("echo line 2 >> {}", output_file.display()), true);
    
    // Check that content was appended, not overwritten
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(
        contents.contains("line 1"),
        "Original content should be preserved"
    );
    assert!(
        contents.contains("line 2"),
        "New content should be appended"
    );
}

#[test]
fn multiple_appends_to_same_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_multiple_appends.txt");
    
    test_case(&format!("echo line 1 >> {}", output_file.display()), true);
    test_case(&format!("echo line 2 >> {}", output_file.display()), true);
    test_case(&format!("echo line 3 >> {}", output_file.display()), true);
    
    // Check that all lines are present in order
    let contents = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    
    assert_eq!(lines.len(), 3, "Should have 3 lines");
    assert_eq!(lines[0], "line 1");
    assert_eq!(lines[1], "line 2");
    assert_eq!(lines[2], "line 3");
}

#[test]
fn append_vs_overwrite_behavior() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_vs_overwrite.txt");
    
    // First write with >>
    test_case(&format!("echo first >> {}", output_file.display()), true);
    // Then overwrite with >
    test_case(&format!("echo second > {}", output_file.display()), true);
    
    // Check that > overwrote the file
    let contents = fs::read_to_string(&output_file).unwrap();
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
    test_case(&format!("echo third >> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(contents.contains("second"), "Second line should remain");
    assert!(contents.contains("third"), "Third line should be appended");
}

#[test]
fn append_empty_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_empty.txt");
    
    // Create file with content
    fs::write(&output_file, "existing content\n").unwrap();
    
    // Append empty echo
    test_case(&format!("echo >> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(
        contents.contains("existing content"),
        "Original content should be preserved"
    );
    // Empty echo adds just a newline
    assert!(contents.len() > "existing content\n".len());
}

#[test]
fn append_with_quoted_filename() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test append file.txt");
    
    test_case(&format!("echo first >> \"{}\"", output_file.display()), true);
    test_case(&format!("echo second >> \"{}\"", output_file.display()), true);
    
    // Check both lines are present
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(contents.contains("first"));
    assert!(contents.contains("second"));
}

#[test]
fn append_external_command_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_external.txt");
    
    // First append
    test_case(&format!("echo '=== PWD ===' >> {}", output_file.display()), true);
    // Append external command output
    test_case(&format!("pwd >> {}", output_file.display()), true);
    // Another append
    test_case(&format!("echo '=== END ===' >> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(contents.contains("=== PWD ==="));
    assert!(contents.contains("=== END ==="));
    // pwd should output something (current directory path)
    let lines: Vec<&str> = contents.lines().collect();
    assert!(lines.len() >= 3, "Should have at least 3 lines");
}

#[test]
fn append_preserves_file_size_growth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_growth.txt");
    
    test_case(&format!("echo a >> {}", output_file.display()), true);
    let size1 = fs::metadata(&output_file).unwrap().len();
    
    test_case(&format!("echo bb >> {}", output_file.display()), true);
    let size2 = fs::metadata(&output_file).unwrap().len();
    
    test_case(&format!("echo ccc >> {}", output_file.display()), true);
    let size3 = fs::metadata(&output_file).unwrap().len();
    
    // Each append should increase file size
    assert!(size2 > size1, "File should grow after second append");
    assert!(size3 > size2, "File should grow after third append");
}

// ========================================================================
// Integration Tests - Append Stderr Redirection (2>>)
// ========================================================================

#[test]
fn append_stderr_to_new_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_stderr_new.txt");
    
    // ls on non-existent directory produces stderr
    test_case(&format!("ls /nonexistent_dir_xyz 2>> {}", output_file.display()), true);
    
    // Check that file was created with stderr
    assert!(
        output_file.exists(),
        "Stderr file should be created with 2>>"
    );
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(
        !contents.is_empty(),
        "Stderr should be captured in file"
    );
    assert!(
        contents.contains("nonexistent_dir_xyz") || contents.contains("No such file"),
        "Error message should mention the missing directory"
    );
}

#[test]
fn append_stderr_to_existing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_stderr_existing.txt");
    
    // Create file with initial content
    fs::write(&output_file, "=== Previous Error ===\n").unwrap();
    
    test_case(&format!("ls /another_nonexistent 2>> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    assert!(
        contents.contains("=== Previous Error ==="),
        "Original content should be preserved"
    );
    assert!(
        contents.contains("another_nonexistent") || contents.contains("No such file"),
        "New stderr should be appended"
    );
}

#[test]
fn append_stderr_multiple_times() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_stderr_multiple.txt");
    
    test_case(&format!("ls /err1 2>> {}", output_file.display()), true);
    test_case(&format!("ls /err2 2>> {}", output_file.display()), true);
    test_case(&format!("ls /err3 2>> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    
    // All three errors should be present
    let has_all = (contents.contains("err1") || contents.matches("No such file").count() >= 1)
        && (contents.contains("err2") || contents.matches("No such file").count() >= 2)
        && (contents.contains("err3") || contents.matches("No such file").count() >= 3);
    
    assert!(has_all, "All three stderr outputs should be appended");
}

#[test]
fn append_stderr_vs_overwrite_stderr() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("test_append_vs_overwrite_stderr.txt");
    
    // First append
    test_case(&format!("ls /error1 2>> {}", output_file.display()), true);
    
    // Then overwrite with 2>
    test_case(&format!("ls /error2 2> {}", output_file.display()), true);
    
    let contents = fs::read_to_string(&output_file).unwrap();
    
    // Should only have error2, not error1 (because 2> overwrites)
    assert!(
        !contents.contains("error1"),
        "First error should be overwritten"
    );
    assert!(
        contents.contains("error2") || contents.contains("No such file"),
        "Second error should be present"
    );
}
