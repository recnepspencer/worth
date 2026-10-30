#[test]
fn raw_layout_bytes_cannot_reopen_the_bootstrap_lane() {
    crate::compile_fail_support::assert_compile_fails_in_ui_dir(
        "bootstrap",
        "raw_persisted_layout_cannot_reopen_bootstrap_lane.rs",
        &["InMemoryPhysicalFormatReplayArtifact", "mismatched types"],
        &["worth_store_physical_format"],
    );
}
