#[test]
fn runtime_authority_compile_fail_boundaries_hold() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/runtime_builder_requires_complete_defaults.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/branch_basis_readmission_authority_is_not_publicly_mintable.rs")).expect("prebuilt compiler denial");
}
