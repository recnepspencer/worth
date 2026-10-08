use super::*;

#[test]
fn genuinely_interleaved_equivalent_sessions_validate_retry_cleanup_separately() {
    let world = installed_authorization_world(true);
    let snapshot_baseline = world.invariant.active_snapshot_count();
    let (left, right) = equivalent_programs(&world, "racing-equivalent");
    let binding = idempotency(139, 140);
    let left = start(&world.application, left, binding);
    let right = start(&world.application, right, binding);
    let left = progress_application_commit(
        &world.application,
        left,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let right = progress_application_commit(
        &world.application,
        right,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    assert_eq!(
        world.invariant.active_snapshot_count(),
        snapshot_baseline + 2
    );
    let left = finish_application_commit(&world.application, left);
    assert_eq!(
        world.invariant.active_snapshot_count(),
        snapshot_baseline + 1,
        "finishing one attempt must preserve its interleaved peer's snapshot lease"
    );
    let right = finish_application_commit(&world.application, right);
    assert_eq!(world.invariant.active_snapshot_count(), snapshot_baseline);
    assert_eq!(world.application.provider_session_resource_count(), 0);

    let (executed, recovered) = match (left, right) {
        (
            WorthQueryApplicationCommitOutcome::Committed(executed),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
        )
        | (
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered),
            WorthQueryApplicationCommitOutcome::Committed(executed),
        ) => (executed, recovered),
        unexpected => panic!("expected one executed and one recovered commit: {unexpected:?}"),
    };
    assert!(executed.is_same_authoritative_commit(&recovered));
    assert_eq!(executed.retained_preimage(), recovered.retained_preimage());
    assert_eq!(executed.dispatch_outbox(), recovered.dispatch_outbox());
    let executed_output = executed
        .outputs_of::<RetentionOutputs>()
        .and_then(|outputs| outputs.entity::<RetainedAccount>())
        .expect("fresh receipt must retain the bound output role");
    let recovered_output = recovered
        .outputs_of::<RetentionOutputs>()
        .and_then(|outputs| outputs.entity::<RetainedAccount>())
        .expect("idempotent receipt must recover the same output role");
    assert_eq!(executed_output.entity_id(), recovered_output.entity_id());
    assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
    assert_eq!(
        recovered.terminal().attempt_resources_released(),
        Some(true)
    );
    assert_eq!(
        executed.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(
        recovered.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
}
