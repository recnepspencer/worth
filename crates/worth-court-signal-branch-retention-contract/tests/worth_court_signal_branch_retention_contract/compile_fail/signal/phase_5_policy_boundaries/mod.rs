#[test]
fn diagnostics_owned_policy_path_is_not_a_second_front_door() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_10/diagnostics_policy_path_is_removed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_10/installed_policy_cannot_be_forged.rs")).expect("prebuilt compiler denial");
}
