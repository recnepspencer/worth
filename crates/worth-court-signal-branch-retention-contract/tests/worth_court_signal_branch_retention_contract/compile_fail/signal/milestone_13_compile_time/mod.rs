#[test]
fn old_frontier_api_is_removed() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_13/old_frontier_api_is_removed.rs")).expect("prebuilt compiler denial");
}

#[test]
fn invalidation_progression_remains_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_13/invalidation_progression_is_private.rs")).expect("prebuilt compiler denial");
}

#[test]
fn predicted_counters_cannot_satisfy_performed_receipt_api() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_13/predicted_counters_are_not_performed.rs")).expect("prebuilt compiler denial");
}
