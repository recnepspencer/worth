use super::*;

#[test]
fn safe_points_project_exact_signal_lifecycle_and_queue_pressure() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let mut basis = runtime
        .admit_managed_execution_basis(
            managed_intent("safe-point-attempt"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime, execution),
            execution,
        )
        .expect("managed execution should admit");
    let available = basis
        .observe_safe_point()
        .expect("active request should produce safe-point evidence");

    assert_eq!(
        available.signal_state(),
        BridgeExecutionSafePointSignalState::Active
    );
    assert_eq!(available.queue_depth(), 0);
    assert_eq!(
        available.pressure_state(),
        BridgeExecutionQueuePressureState::Available
    );
    assert_eq!(available.observation_ordinal(), 0);
    assert_eq!(available.counters().exact_signal_request_lookup_count(), 1);
    assert_eq!(available.counters().pressure_classification_count(), 1);

    let admission = basis
        .enqueue_managed_queue(4)
        .expect("exact bridge basis should fill its Signal-owned queue");
    let saturation = admission.mutation();
    assert_eq!(
        saturation.pressure_state(),
        BridgeExecutionQueuePressureState::Saturated
    );
    assert_eq!(saturation.queue_depth(), 4);
    with_async_request_signal_runtime(runtime.signal_runtime_key, |signal_runtime| {
        signal_runtime.cancel_resource_request(
            basis.request().request_handle(),
            ResourceCancellationReason::HostRequested,
        )
    })
    .expect("test runtime should stay on one thread")
    .expect("exact request cancellation should succeed");
    let saturated = basis
        .observe_safe_point()
        .expect("cancelled request should retain observable lifecycle evidence");
    assert_eq!(
        saturated.signal_state(),
        BridgeExecutionSafePointSignalState::Cancelled
    );
    assert_eq!(
        saturated.pressure_state(),
        BridgeExecutionQueuePressureState::Saturated
    );
    assert_eq!(saturated.observation_ordinal(), 1);
    let (_, occupancy) = admission.into_parts();
    let release = basis
        .release_managed_queue_occupancy(occupancy)
        .expect("terminal Signal requests must still release exact queue occupancy");
    assert_eq!(release.queue_depth(), 0);
}

#[test]
fn queue_overflow_denies_without_mutating_signal_pressure() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let mut basis = runtime
        .admit_managed_execution_basis(
            managed_intent("queue-contract-attempt"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime, execution),
            execution,
        )
        .expect("managed execution should admit");

    let denial = basis
        .enqueue_managed_queue(5)
        .expect_err("the bound queue capacity must reject overflow");
    assert_eq!(
        denial.kind(),
        BridgeManagedQueueFailureKind::SignalQueueMutationDenied
    );
    let first_valid = basis
        .observe_safe_point()
        .expect("overflow denial should preserve the bound queue");
    assert_eq!(first_valid.observation_ordinal(), 0);
    assert_eq!(first_valid.queue_depth(), 0);
    assert_eq!(first_valid.queue_capacity(), 4);

    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .expect("test basis should clean up");
}

#[test]
fn execution_basis_cannot_finalize_while_queue_occupancy_is_outstanding() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let mut basis = runtime
        .admit_managed_execution_basis(
            managed_intent("queue-finalization-attempt"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime, execution),
            execution,
        )
        .expect("managed execution should admit");
    let admission = basis
        .enqueue_managed_queue(1)
        .expect("one queue unit should admit");
    let (_, occupancy) = admission.into_parts();

    let failure = basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Abandoned)
        .expect_err("outstanding queue occupancy must retain the execution basis");
    assert_eq!(
        failure.kind(),
        BridgeExecutionBasisFinalizationFailureKind::ManagedQueueOccupied
    );
    let mut basis = failure.into_basis();
    basis
        .release_managed_queue_occupancy(occupancy)
        .expect("exact occupancy should remain releasable");
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Abandoned)
        .expect("released queue occupancy should permit finalization");
}
