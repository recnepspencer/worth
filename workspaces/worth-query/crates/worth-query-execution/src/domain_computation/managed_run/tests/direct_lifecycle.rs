use super::*;

#[test]
fn causal_lower_authorities_admit_one_managed_direct_run_and_cleanup_every_owner() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = query_runtime();
    let plan = admitted_plan("managed-direct", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact installed operation should start one reserved attempt");
    let lower = causal_fixture::managed_admission_context();

    let admitted = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("managed admission should compose its lower authorities");
    assert_eq!(admitted.counters().query_runtime_check_count(), 1);
    assert_eq!(admitted.counters().resource_attempt_check_count(), 1);
    assert_eq!(admitted.counters().bridge_intent_check_count(), 1);
    assert_eq!(admitted.counters().bridge_source_check_count(), 1);
    assert_eq!(admitted.counters().relational_basis_check_count(), 1);
    assert_eq!(admitted.counters().semantic_basis_check_count(), 1);

    let terminal = admitted
        .start()
        .completed()
        .expect("a run with no unverified provider work may complete");
    assert_eq!(terminal.kind(), WorthQueryManagedRunTerminalKind::Completed);
    let cleanup = terminal
        .cleanup()
        .expect("owner-thread completion cleanup should succeed");

    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
    assert!(cleanup.inspection().resources_released());
    assert_eq!(
        cleanup.inspection().capacity_scope(),
        WorthQueryExecutionCapacityReservationScope::Direct
    );
    assert_eq!(cleanup.inspection().released_reservation_count(), 1);
}

#[test]
fn cleanup_thread_failure_returns_all_authority_for_owner_thread_retry() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = query_runtime();
    let plan = admitted_plan("cleanup-retry", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact operation should start");
    let lower = causal_fixture::managed_admission_context();
    let terminal = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("managed run should admit")
        .start()
        .completed()
        .expect("empty run should complete");

    let failure = std::thread::spawn(move || {
        terminal
            .cleanup()
            .expect_err("foreign thread must not terminalize Signal")
    })
    .join()
    .expect("cleanup probe should return its recovery authority");

    assert_eq!(
        failure.failure_kind(),
        WorthQueryManagedRunCleanupFailureKind::BridgeFinalization(
            BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation
        )
    );
    assert_eq!(
        failure.disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
    let cleanup = failure
        .retry()
        .expect("Signal owner thread should complete the same cleanup");
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
    assert!(cleanup.inspection().resources_released());
    assert_eq!(cleanup.inspection().released_reservation_count(), 1);
}
