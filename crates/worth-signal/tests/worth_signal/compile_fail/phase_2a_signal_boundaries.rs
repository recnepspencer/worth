#[test]
fn signal_tokens_and_digests_reject_authority_artifact_apis() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/phase_2a/*.rs");
}
