#[cfg(feature = "recovery-runtime-owner")]
#[test]
fn recovered_core_cannot_issue_two_serving_custody_seals() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(
        "tests/recovered_custody_authority/recovered_core_cannot_issue_two_seals.rs",
    );
}
