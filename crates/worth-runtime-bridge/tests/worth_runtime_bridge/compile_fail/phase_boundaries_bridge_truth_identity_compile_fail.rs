#[test]
fn bridge_truth_identity_string_facade_is_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/bridge_truth_identity/*.rs");
}
