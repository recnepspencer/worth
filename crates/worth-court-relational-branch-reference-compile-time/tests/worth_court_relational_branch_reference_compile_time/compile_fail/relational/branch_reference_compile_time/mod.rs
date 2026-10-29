#[test]
fn relational_branch_authority_is_owner_sealed() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/branch_reference/*.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/branch_reference_pass/*.rs"));
}

#[test]
fn relational_change_receipts_are_owner_minted() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source/*.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source_pass/*.rs"));
}
