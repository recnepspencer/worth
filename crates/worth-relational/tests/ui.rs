#[test]
fn invariant_ui_boundaries_are_sealed() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/invariants/*.rs");
}

#[test]
fn a_durable_medium_is_moved_into_exactly_one_runtime() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/durable_medium/*.rs");
    cases.pass("tests/ui/durable_medium_pass/*.rs");
}
