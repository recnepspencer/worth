#[allow(unused_imports)]
pub(crate) use crate::fast::phase_5_policy_boundaries::*;

#[test]
fn diagnostics_owned_policy_path_is_not_a_second_front_door() {
    let cases = TestCases::new();
    cases.compile_fail("tests/ui/milestone_10/diagnostics_policy_path_is_removed.rs");
    cases.compile_fail("tests/ui/milestone_10/installed_policy_cannot_be_forged.rs");
}
