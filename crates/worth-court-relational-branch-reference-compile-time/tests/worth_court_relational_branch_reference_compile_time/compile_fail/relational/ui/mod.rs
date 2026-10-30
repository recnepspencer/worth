#[test]
fn invariant_ui_boundaries_are_sealed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/invariants/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn a_durable_medium_is_moved_into_exactly_one_runtime() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/durable_medium/*.rs")).expect("prebuilt compiler denial");
}
