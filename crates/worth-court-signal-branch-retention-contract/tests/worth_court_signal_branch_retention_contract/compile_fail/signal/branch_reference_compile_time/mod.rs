#[test]
fn signal_branch_authority_is_owner_sealed() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui-pass/branch_reference/*.rs");
    cases.compile_fail("tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/branch_reference/*.rs");
}
