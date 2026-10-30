#[test]
fn relational_branch_authority_is_owner_sealed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/branch_reference/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn relational_change_receipts_are_owner_minted() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source/*.rs")).expect("prebuilt compiler denial");
}
