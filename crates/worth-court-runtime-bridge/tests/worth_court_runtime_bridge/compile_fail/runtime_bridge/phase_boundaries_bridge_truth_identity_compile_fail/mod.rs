#[test]
fn bridge_truth_identity_string_facade_is_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/*.rs")).expect("prebuilt compiler denial");
}
