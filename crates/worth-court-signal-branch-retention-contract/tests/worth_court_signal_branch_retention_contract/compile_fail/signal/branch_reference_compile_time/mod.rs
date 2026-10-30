#[test]
fn signal_branch_authority_is_owner_sealed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/branch_reference/*.rs")).expect("prebuilt compiler denial");
}
