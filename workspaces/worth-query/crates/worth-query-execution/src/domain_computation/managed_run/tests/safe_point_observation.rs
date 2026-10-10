use worth_runtime_bridge::facade::{
    BridgeExecutionQueuePressureState, BridgeExecutionSafePointSignalState,
};

use super::*;
#[test]
fn direct_run_observes_signal_and_pressure_through_its_bound_bridge_basis() {
    let runtime = query_runtime();
    let plan = admitted_plan("direct-safe-point", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("direct safe-point attempt should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("direct safe-point run should admit")
        .start();

    let available = running
        .observe_safe_point()
        .expect("active direct run should observe one safe point");
    assert_eq!(available.run_identity(), running.identity());
    assert_eq!(
        available.signal_state(),
        BridgeExecutionSafePointSignalState::Active
    );
    assert_eq!(
        available.pressure_state(),
        BridgeExecutionQueuePressureState::Available
    );
    assert_eq!(available.queue_depth(), 0);
    assert_eq!(available.queue_capacity(), 8);
    assert_eq!(available.observation_ordinal(), 0);
    assert_eq!(available.counters().exact_signal_request_lookup_count(), 1);

    let repeated = running
        .observe_safe_point()
        .expect("safe-point observation should read the same Signal-owned queue");
    assert_eq!(
        repeated.pressure_state(),
        BridgeExecutionQueuePressureState::Available
    );
    assert_eq!(repeated.queue_depth(), 0);
    assert_eq!(repeated.observation_ordinal(), 1);
    running
        .terminal(WorthQueryManagedRunTerminalKind::Cancelled)
        .cleanup()
        .expect("safe-point run should clean up");
}

#[test]
fn workflow_run_uses_the_same_managed_safe_point_authority() {
    let runtime = query_runtime();
    let operation_resources = admitted_plan("workflow-safe-point", 8);
    let stage_resources = admitted_plan("workflow-safe-point:stage", 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority(&runtime, &resources);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("workflow safe-point attempt should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request())
        .expect("workflow safe-point run should admit")
        .start()
        .expect("workflow artifact authority should start");

    let observation = running
        .observe_safe_point()
        .expect("workflow run should consume the shared safe-point path");
    assert_eq!(observation.run_identity(), running.identity());
    assert_eq!(
        observation.pressure_state(),
        BridgeExecutionQueuePressureState::Available
    );
    assert_eq!(observation.queue_depth(), 0);
    assert_eq!(observation.queue_capacity(), 8);
    match running
        .terminal(WorthQueryManagedRunTerminalKind::Cancelled)
        .cleanup()
    {
        WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
        WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
            panic!("empty workflow safe-point run retained an artifact owner")
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
            panic!("empty workflow safe-point cleanup failed: {failure:?}")
        }
    }
}

#[test]
fn repeated_safe_point_observation_has_no_heap_allocation() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_managed_safe_point_allocation_probe")
        .env("WORTH_QUERY_SAFE_POINT_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "safe-point allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn isolated_managed_safe_point_allocation_probe() {
    if std::env::var_os("WORTH_QUERY_SAFE_POINT_ALLOCATION_PROBE").is_none() {
        return;
    }
    let runtime = query_runtime();
    let plan = admitted_plan("direct-safe-point-allocation", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("allocation probe attempt should reserve");
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("allocation probe run should admit")
        .start();

    let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
    for _ in 0..64 {
        let observation = running
            .observe_safe_point()
            .expect("repeated safe-point observation should remain admitted");
        std::hint::black_box(observation.observation_ordinal());
    }
    let stats = region.change();
    assert_eq!(stats.allocations, 0, "{stats:?}");
    assert_eq!(stats.bytes_allocated, 0, "{stats:?}");

    running
        .terminal(WorthQueryManagedRunTerminalKind::Cancelled)
        .cleanup()
        .expect("allocation probe run should clean up");
}
