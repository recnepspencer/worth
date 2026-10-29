#[allow(unused_imports)]
pub(crate) use {trybuild::TestCases};

#[test]
fn diagnostics_owned_policy_path_is_not_a_second_front_door() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_10/diagnostics_policy_path_is_removed.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_10/installed_policy_cannot_be_forged.rs"));
}
