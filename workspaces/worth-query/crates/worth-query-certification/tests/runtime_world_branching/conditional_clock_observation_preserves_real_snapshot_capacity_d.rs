//! conditional clock observation preserves real snapshot capacity denial.
use super::*;

#[test]
fn conditional_clock_observation_preserves_real_snapshot_capacity_denial() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    const MAXIMUM_ACTIVE_SNAPSHOTS: usize = 3;
    let world =
        CourtroomWorld::publish_with_active_snapshot_limit("ready", MAXIMUM_ACTIVE_SNAPSHOTS);
    let (_, before) = world
        .application
        .relational_snapshot_state_for_test(request_execution);
    let branch = world.application.current_world();
    let selected = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(branch)
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let already_active = world
        .application
        .relational_snapshot_state_for_test(request_execution)
        .0;
    let pinned = (already_active..MAXIMUM_ACTIVE_SNAPSHOTS)
        .map(|_| {
            world
                .application
                .with_host_advancement(|host_phase| {
                    world
                        .application
                        .on_branch(branch)
                        .select(&host_phase)
                        .expect("each basis inside the configured limit must be admitted")
                })
                .expect("installed host call opens")
        })
        .collect::<Vec<_>>();
    let retained_after_failed_admission = pinned.len();
    assert_eq!(
        world
            .application
            .relational_snapshot_state_for_test(request_execution)
            .0,
        MAXIMUM_ACTIVE_SNAPSHOTS
    );

    let denial = match selected.conditional_clock(request_execution, &world.clock) {
        Ok(_) => panic!("capacity exhaustion must deny public product admission"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalClockObservationDenialKind::ProductAdmission(
            primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: MAXIMUM_ACTIVE_SNAPSHOTS,
            },
        )
    );
    assert_eq!(world.contacts.snapshot(), (0, 0, 0, 0));
    assert_eq!(
        world
            .application
            .relational_snapshot_state_for_test(request_execution)
            .0,
        retained_after_failed_admission,
        "failed selected-product admission must release its own retained truth"
    );
    drop(pinned);
    let (active, after) = world
        .application
        .relational_snapshot_state_for_test(request_execution);
    assert_eq!(active, 0);
    assert_eq!(after, before);
    let retry = world.conditional_clock().observe(request_execution);
    assert!(matches!(
        retry,
        primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    assert_eq!(
        world
            .application
            .relational_snapshot_state_for_test(request_execution)
            .0,
        0
    );
}
