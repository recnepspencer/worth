use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
#[test]
fn signal_branch_authority_is_owner_sealed() {
    worth_testkit_compiler::production_denials(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/branch_reference/*.rs"), &[Contract { fixture: "managed_reference_cannot_deserialize.rs", errors: &[Diagnostic::Code { code: "E0277", symbols: &["ManagedSignalBranchReference", "Deserialize"] }] }]).expect("production compiler denial");
}
