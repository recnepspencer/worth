use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
#[test]
fn relational_branch_authority_is_owner_sealed() {
    worth_testkit_compiler::production_denials(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/branch_reference/*.rs"), &[Contract { fixture: "admitted_basis_cannot_be_fabricated.rs", errors: &[Diagnostic::Code { code: "E0451", symbols: &["AdmittedRelationalBranchBasis", "inner"] }, Diagnostic::Uncoded { cause: "is private", symbols: &["AdmittedRelationalBranchBasisInner"] }] }]).expect("production compiler denial");
}

#[test]
fn relational_change_receipts_are_owner_minted() {
    worth_testkit_compiler::production_denials(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source/*.rs"), &[Contract { fixture: "receipt_cannot_be_fabricated.rs", errors: &[Diagnostic::Uncoded { cause: "cannot construct", symbols: &["RelationalChangeReceipt", "private fields"] }] }]).expect("production compiler denial");
}
