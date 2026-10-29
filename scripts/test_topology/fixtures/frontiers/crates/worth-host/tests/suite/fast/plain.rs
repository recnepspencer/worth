#[test]
fn ordinary_value_uses_shared_support() {
    assert_eq!(worth_host::host_value(), 2);
    assert_eq!(crate::support::shared_value(), 7);
    assert_eq!(include_str!("../../fixtures/message.txt"), "fixture\n");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/message.txt");
    assert!(path.exists());
}
