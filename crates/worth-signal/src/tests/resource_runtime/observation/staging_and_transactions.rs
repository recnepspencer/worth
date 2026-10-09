use super::*;

#[test]
fn resource_completion_stage_and_commit_apply_lifecycle_once() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");
    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let handle = admitted_request.handle();
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

    let staging = runtime
        .stage_admitted_resource_completion(admitted_completion)
        .expect("active admitted completion should stage");
    assert_eq!(
        staging.performance().boundary(),
        ResourceBoundaryKind::CompletionStaging
    );
    assert_eq!(staging.performance().cost_contract().get(), 9);
    assert_eq!(
        staging.performance().cost_posture(),
        ResourceCostPosture::Verified
    );
    assert_eq!(staging.performance().lifecycle_transition_count(), 0);
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("staging must not mutate request lifecycle")
            .status(),
        ResourceInFlightStatus::Active
    );
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);

    let commit = runtime
        .commit_staged_resource_completion(staging.staged_effect())
        .expect("staged completion should commit exactly once");

    assert_eq!(
        commit.performance().boundary(),
        ResourceBoundaryKind::CompletionCommit
    );
    assert_eq!(commit.lifecycle().node(), ResourceNodeId::from_node(node));
    assert_eq!(
        commit.lifecycle().lifecycle(),
        ResourceLifecycleClass::Fulfilled
    );
    assert_eq!(
        commit.transition().kind(),
        ResourceLifecycleTransitionKind::CompletionAdmitted
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("fulfilled request remains retained for audit")
            .status(),
        ResourceInFlightStatus::Fulfilled
    );
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .active_in_flight_node_count(),
        0
    );
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 0);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 1);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_completion_staging_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_completion_commit_count,
        1
    );
}

#[test]
fn resource_completion_rollback_of_staged_admitted_preserves_pending_request() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(timeout_resource_declaration(node, 5))
        .expect("resource declaration should lower");
    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let handle = admitted_request.handle();
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

    let staged = runtime
        .stage_admitted_resource_completion(admitted_completion)
        .expect("active admitted completion should stage")
        .staged_effect();
    let rollback = runtime.rollback_staged_resource_completion(staged);

    assert_eq!(
        rollback.performance().boundary(),
        ResourceBoundaryKind::CompletionRollback
    );
    assert_eq!(rollback.performance().admitted_count(), 1);
    assert_eq!(rollback.performance().denied_count(), 0);
    assert_eq!(rollback.performance().lifecycle_transition_count(), 0);
    assert_eq!(
        rollback.rolled_back_completion().subject(),
        ResourceCompletionRollbackSubject::Admitted {
            handle,
            node: ResourceNodeId::from_node(node),
            completion_ordinal: ResourceCompletionOrdinal::new(1),
        }
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("rollback must leave request available for a later commit")
            .status(),
        ResourceInFlightStatus::Active
    );
    assert_eq!(
        runtime
            .resource_runtime_summary()
            .active_in_flight_node_count(),
        1
    );
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 0);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_completion_rollback_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_completion_commit_count,
        0
    );
}

#[test]
fn resource_completion_transaction_commit_delivers_lifecycle_observation_once() {
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
    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let handle = admitted_request.handle();
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
    let calls = Arc::new(Mutex::new(Vec::<ResourceObservationRecord>::new()));
    let observation_handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let result = runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staging = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staging.staged_effect())?;
            Ok(())
        })
        .expect("completion transaction should commit");

    let recorded = calls
        .lock()
        .expect("resource observation mutex poisoned")
        .clone();
    assert_eq!(
        recorded,
        vec![ResourceObservationRecord {
            observer_id: observation_handle.observer_id().get(),
            handle_id: observation_handle.handle_id().get(),
            matched_node_count: 1,
            visited: true,
            recomputed: false,
            meaningful_change: true,
            trigger_matched: true,
        }]
    );
    assert_eq!(result.observation.classified_event_count, 1);
    assert_eq!(result.observation.trigger_matched_event_count, 1);
    assert_eq!(result.observation.delivered_event_count, 1);
    assert_eq!(result.observation.rollback_suppressed_event_count, 0);
    assert_eq!(
        result.observation.boundary_events[0]
            .matched_nodes
            .iter()
            .collect::<Vec<_>>(),
        vec![node]
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("fulfilled request remains retained for audit")
            .status(),
        ResourceInFlightStatus::Fulfilled
    );
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 0);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 1);
    assert_eq!(
        runtime.telemetry().transaction.delivered_observation_count,
        1
    );
}

#[test]
fn resource_completion_transaction_rollback_suppresses_observation_and_restores_state() {
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
    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let handle = admitted_request.handle();
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
    let calls = Arc::new(Mutex::new(Vec::<ResourceObservationRecord>::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    let staging = tx
        .stage_admitted_resource_completion(admitted_completion)
        .expect("completion should stage inside transaction");
    tx.commit_staged_resource_completion(staging.staged_effect())
        .expect("completion should mutate transaction-local resource state");
    let result = tx
        .rollback()
        .expect("rollback should restore resource and temporal state");

    assert!(
        calls
            .lock()
            .expect("resource observation mutex poisoned")
            .is_empty(),
        "rollback must suppress completion-driven observation delivery"
    );
    assert_eq!(result.observation.classified_event_count, 1);
    assert_eq!(result.observation.trigger_matched_event_count, 1);
    assert_eq!(result.observation.delivered_event_count, 0);
    assert_eq!(result.observation.rollback_suppressed_event_count, 1);
    assert!(matches!(
        result.observation.boundary_events[0].outcome,
        ObservationBoundaryOutcome::RollbackSuppressed
    ));
    assert_eq!(
        runtime
            .in_flight_resource_request(handle)
            .expect("rollback must restore active request")
            .status(),
        ResourceInFlightStatus::Active
    );
    assert_eq!(runtime.temporal_wake_summary().scheduled_count(), 1);
    assert_eq!(runtime.temporal_wake_summary().retired_count(), 0);
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_resource_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_packet_temporal_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .transaction
            .rollback_suppressed_observation_count,
        1
    );
}
