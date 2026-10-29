#[test]
fn lineage_phase_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/lineage/*.rs"));
}

#[test]
fn merge_phase_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/merge/*.rs"));
}

#[test]
fn prepared_candidate_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/prepared_candidate/*.rs"));
}
