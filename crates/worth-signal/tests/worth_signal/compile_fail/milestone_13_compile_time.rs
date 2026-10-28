#[test]
fn phase_six_public_invalidation_observation_surface_is_current() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/milestone_13/frontier_public_inventory.rs");
}

#[test]
fn phase_six_old_frontier_api_is_removed() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/milestone_13/old_frontier_api_is_removed.rs");
}

#[test]
fn phase_two_invalidation_progression_remains_private() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/milestone_13/invalidation_progression_is_private.rs");
}

#[test]
fn phase_six_predicted_counters_cannot_satisfy_performed_receipt_api() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/milestone_13/predicted_counters_are_not_performed.rs");
}
