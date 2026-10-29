#[test]
fn merge_api_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/runtime_builder_requires_complete_defaults.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/lowered_merge_plan_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/branch_basis_readmission_authority_is_not_publicly_mintable.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/merge_strategy_witness_cannot_be_deserialized.rs"));
}
