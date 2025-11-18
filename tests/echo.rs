mod common;
use common::test_case;

#[test]
fn echo_prints_hello_world() {
    let output = test_case("echo hello world", true);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("hello world"));
    assert_eq!(output.status.code(), Some(0));
}
