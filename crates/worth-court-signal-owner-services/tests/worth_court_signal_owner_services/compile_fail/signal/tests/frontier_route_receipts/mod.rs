#[test]
fn frontier_route_receipt_compile_fail_boundaries_hold() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/frontier_route_evidence_receipt_fields_are_private.rs")).expect("prebuilt compiler denial");
}
