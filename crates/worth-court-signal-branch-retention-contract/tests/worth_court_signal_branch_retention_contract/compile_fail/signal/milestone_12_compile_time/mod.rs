#[test]
fn invalidation_authority_forms_are_not_publicly_mintable() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_12/*.rs"));
}
