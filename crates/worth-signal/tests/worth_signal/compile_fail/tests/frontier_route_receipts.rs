#[test]
fn frontier_route_receipt_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/frontier_route_evidence_receipt_fields_are_private.rs");
}
