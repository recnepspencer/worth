//! Toolchain identity is a pure value check.
#[test]
fn world_artifact_name_copy_uses_the_reviewed_committed_compiler() {
    let toolchain = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml"),
    )
    .unwrap();
    assert_eq!(
        toolchain.lines().find(|line| line.starts_with("channel")),
        Some("channel = \"1.99.1\"")
    );
}
