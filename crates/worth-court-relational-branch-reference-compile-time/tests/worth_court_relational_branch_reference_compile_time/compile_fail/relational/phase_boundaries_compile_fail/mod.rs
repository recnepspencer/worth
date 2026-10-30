#[test]
fn lineage_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/lineage/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn merge_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/merge/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn prepared_candidate_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/prepared_candidate/*.rs")).expect("prebuilt compiler denial");
}
