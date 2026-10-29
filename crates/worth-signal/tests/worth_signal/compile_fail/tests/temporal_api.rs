#[test]
fn temporal_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/temporal_duration_fields_are_private.rs");
    cases.compile_fail("tests/ui/after_condition_fields_are_private.rs");
    cases.compile_fail("tests/ui/scheduled_temporal_wake_fields_are_private.rs");
    cases.compile_fail("tests/ui/ready_temporal_wake_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_frontier_snapshot_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_clock_advance_summary_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_ready_promotion_summary_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_previous_value_access_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_previous_value_reference_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_wake_reschedule_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_wake_admission_summary_fields_are_private.rs");
    cases.compile_fail("tests/ui/interval_wake_regeneration_fields_are_private.rs");
    cases.compile_fail("tests/ui/temporal_wake_retirement_batch_fields_are_private.rs");
    cases.compile_fail("tests/ui/lowered_temporal_eligibility_fields_are_private.rs");
}
