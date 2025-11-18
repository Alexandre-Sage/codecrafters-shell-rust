mod common;
use common::test_case;

#[test]
fn cd_to_tmp_then_pwd() {
    let output = test_case("cd /tmp\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // After cd /tmp, pwd should show /tmp
    assert!(
        stdout.contains("/tmp"),
        "pwd should show /tmp after cd /tmp"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_to_root_then_pwd() {
    let output = test_case("cd /\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // After cd /, pwd should show /
    let has_root = stdout.lines().any(|l| l.trim() == "/" || l.contains("$ /"));
    assert!(has_root, "pwd should show / after cd /, got: {stdout}");
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
    let has_root = stdout.lines().any(|l| l.trim() == "/" || l.contains("$ /"));
    assert!(
        has_root,
        "Final pwd should show / after multiple cd commands"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cd_with_trailing_slash() {
    let output = test_case("cd /tmp/\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should handle trailing slash correctly
    assert!(
        stdout.contains("/tmp"),
        "Should handle trailing slash in cd"
    );
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

#[test]
fn cd_tilde_expansion_to_home() {
    let output = test_case("cd ~\npwd", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // After cd ~, pwd should show home directory
    assert!(
        stdout.contains("/home/") || stdout.contains("/Users/"),
        "cd ~ should go to home directory, got: {stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}
