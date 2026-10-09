use super::*;

pub(in crate::tests::resource_runtime) fn resource_async_lifecycle_rollback_workload(
) -> ResourceAsyncLifecycleRollbackWorkloadOutcome {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");
    let calls = Arc::new(Mutex::new(Vec::<ResourceObservationRecord>::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let handle = admitted_request.handle();
    let pre_rollback_replay = runtime.reconstruct_resource_replay_summary();
    let admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            handle,
            admitted_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("matching completion should admit");

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    let staging = tx
        .stage_admitted_resource_completion(admitted_completion)
        .expect("completion should stage inside transaction");
    tx.commit_staged_resource_completion(staging.staged_effect())
        .expect("completion should mutate transaction-local resource state");
    tx.rollback()
        .expect("rollback should restore resource and temporal state");

    let rollback_observation = runtime
        .latest_resource_observation_batch_report()
        .expect("rollback should publish a suppressed observation packet");
    let delivered_observations_after_rollback = calls
        .lock()
        .expect("resource observation mutex poisoned")
        .clone();
    let post_rollback_replay = runtime.reconstruct_resource_replay_summary();
    let diagnostics_after_rollback =
        runtime.resource_diagnostics_summary_with_unbounded_cold_reconstruction();
    let rollback_admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            handle,
            admitted_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("restored active request should admit a second completion proof");
    let rollback_staging = runtime
        .stage_admitted_resource_completion(rollback_admitted_completion)
        .expect("runtime rollback completion should stage");
    let rollback_report =
        runtime.rollback_staged_resource_completion(rollback_staging.staged_effect());

    let committed_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            handle,
            admitted_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("same completion should still admit after rollback");
    let mut control_ctx = ();
    let mut control_tx = runtime.begin(request_execution, &mut control_ctx);
    let committed_staging = control_tx
        .stage_admitted_resource_completion(committed_completion)
        .expect("post-rollback control completion should stage");
    control_tx
        .commit_staged_resource_completion(committed_staging.staged_effect())
        .expect("post-rollback control completion should mutate transaction-local state");
    control_tx
        .commit()
        .expect("post-rollback control completion transaction should commit");
    let control_commit_observation = runtime
        .latest_resource_observation_batch_report()
        .expect("control commit should publish a delivered observation packet");
    let delivered_observations_after_control_commit = calls
        .lock()
        .expect("resource observation mutex poisoned")
        .clone();
    let control_path_replay = runtime.reconstruct_resource_replay_summary();

    ResourceAsyncLifecycleRollbackWorkloadOutcome {
        pre_rollback_replay,
        post_rollback_replay,
        control_path_replay,
        diagnostics_after_rollback,
        rollback_report,
        rollback_observation,
        control_commit_observation,
        delivered_observations_after_rollback,
        delivered_observations_after_control_commit,
    }
}
