use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
#[test]
fn lineage_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::production_denials(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/lineage/*.rs"), &[Contract { fixture: "lineage_finalization_artifact_private.rs", errors: &[Diagnostic::Code { code: "E0432", symbols: &["LineageFinalizationArtifact", "facade::lineage"] }] }]).expect("production compiler denial");
}

#[test]
fn merge_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/merge/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn prepared_candidate_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/prepared_candidate/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn memory_cutover_cannot_escape_or_perform_twice() {
    worth_testkit_compiler::production_denial(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/scoped_cutover/*.rs")).expect("prebuilt scoped cutover denial");
}
