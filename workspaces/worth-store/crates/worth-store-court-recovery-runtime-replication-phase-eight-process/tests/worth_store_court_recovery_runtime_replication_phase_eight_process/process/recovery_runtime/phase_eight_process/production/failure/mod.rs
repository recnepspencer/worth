use {crate::phase_eight_process::production::harness};

#[test]
fn observer_missing_root_fails_without_emitting_a_report() {
    crate::phase_eight_process::production::harness::assert_missing_root_observer_fails();
}
