#[test]
fn subscription_replay_tests_reject_label_fixture_mints() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/subscription/replay_tests.rs"
    ))
    .expect("replay_tests.rs should be readable");
    for pattern in [
        "truth_snapshot_fixture(",
        "truth_branch_fixture(",
        "truth_commit_fixture(",
        "truth_patch_fixture(",
    ] {
        assert!(
            !source.contains(pattern),
            "subscription replay tests must not mint truth identity from label fixtures (`{pattern}`)"
        );
    }
}
