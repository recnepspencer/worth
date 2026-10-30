use crate::support as shared;
#[test]
fn ordinary_value_uses_shared_support() {
    assert_eq!(worth_host::host_value(), 2);
    assert_eq!(shared::shared_value(), 7);
    struct Snapshot {
        amount: u32,
    }
    let actual = Snapshot {
        amount: crate::support::shared_value(),
    };
    assert_eq!(actual.amount, 7);
    assert_eq!(include_str!("../../fixtures/message.txt"), "fixture\n");
    let executable = env!("CARGO_BIN_EXE_host-inspect");
    assert!(!executable.is_empty());
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/message.txt");
    assert!(path.exists());
}

pub(crate) fn cross_category_value() -> u32 {
    3
}
