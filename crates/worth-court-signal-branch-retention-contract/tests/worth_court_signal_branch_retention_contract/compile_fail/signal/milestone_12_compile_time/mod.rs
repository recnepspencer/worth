#[test]
fn invalidation_authority_forms_are_not_publicly_mintable() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_12/*.rs")).expect("prebuilt compiler denial");
}
