#[test]
fn lineage_phase_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/lineage/*.rs");
}

#[test]
fn merge_phase_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/merge/*.rs");
}

#[test]
fn prepared_candidate_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/prepared_candidate/*.rs");
}
