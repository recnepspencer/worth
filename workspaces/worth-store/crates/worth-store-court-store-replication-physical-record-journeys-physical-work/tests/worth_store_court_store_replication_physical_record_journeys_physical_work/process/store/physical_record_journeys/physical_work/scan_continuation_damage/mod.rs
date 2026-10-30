#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::scan_continuation_damage::*;

#[test]
fn continuation_damage_revokes_shared_health_and_fences_mutation() {
    let calibration = calibrate();
    let (parent, serving) = open_faulted(calibration);
    let (before, invalidations_before, causal_start) =
        fail_first_continuation(&serving, calibration);
    await_signal_cleanup(&serving);
    assert_failed_effect(&serving, before, invalidations_before);
    assert_failed_scan_route(&serving, causal_start);
    assert_mutation_fenced(&serving);
    assert!(serving.close_plan().execute().requires_inspection());
    drop(parent);
}
