#[test]
fn signal_tokens_and_digests_reject_authority_artifact_apis() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/phase_2a/*.rs"
    ))
    .expect("prebuilt compiler denial");
}
