#[test]
fn bridge_truth_identity_string_facade_is_private() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/*.rs");
}
