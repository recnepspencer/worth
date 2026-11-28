#[test]
fn stock_managed_index_rejects_refinement() {
    trybuild::TestCases::new().compile_fail("tests/ui/stock_index_refinement.rs");
}
