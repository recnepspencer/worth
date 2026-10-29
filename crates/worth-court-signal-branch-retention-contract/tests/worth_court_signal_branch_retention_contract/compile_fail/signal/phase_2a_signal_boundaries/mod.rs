#[test]
fn signal_tokens_and_digests_reject_authority_artifact_apis() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/phase_2a/*.rs"));
}
