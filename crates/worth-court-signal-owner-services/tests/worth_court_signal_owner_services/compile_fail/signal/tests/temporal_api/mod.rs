#[test]
fn temporal_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_duration_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/after_condition_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/scheduled_temporal_wake_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/ready_temporal_wake_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_frontier_snapshot_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_clock_advance_summary_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_ready_promotion_summary_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_previous_value_access_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_previous_value_reference_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_wake_reschedule_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_wake_admission_summary_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/interval_wake_regeneration_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/temporal_wake_retirement_batch_fields_are_private.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/lowered_temporal_eligibility_fields_are_private.rs"));
}
