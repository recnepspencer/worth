#[test]
fn signal_branch_authority_is_owner_sealed() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui-pass/branch_reference/*.rs");
    cases.compile_fail("tests/ui/branch_reference/*.rs");
}
