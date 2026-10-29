pub(crate) use worth_query_worlds::facade::certification::application_graph::recorded_replay_ordering::lookup_growth::*;

/// A growth class, not a count: thirty-two times the recorded outcomes must
/// not make the World-side pre-admission step grow with them. Samples of the
/// two hosts alternate, so machine load lands on both, and each side keeps its
/// fastest sample, since noise only adds. A step that walked the records would
/// scale toward 32x; the bound leaves room for noise without admitting that.
#[test]
fn the_pre_admission_step_does_not_grow_with_recorded_outcomes() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(0, stopping.token()))
        .unwrap();
    stopping.cancel();
    let small = populated(4);
    let large = populated(128);
    let (mut small_floor, mut large_floor) = (Duration::MAX, Duration::MAX);
    for _ in 0..REPLAY_SAMPLES {
        small_floor = small_floor.min(timed_replay(&small, &stopped));
        large_floor = large_floor.min(timed_replay(&large, &stopped));
    }
    assert!(
        large_floor <= small_floor.saturating_mul(4),
        "replay floor grew from {small_floor:?} at 4 recorded outcomes to {large_floor:?} at 128"
    );
}
