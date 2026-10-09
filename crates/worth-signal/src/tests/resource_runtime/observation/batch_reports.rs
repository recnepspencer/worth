use super::*;

#[test]
fn resource_observation_batch_report_respects_lifecycle_only_and_output_policies_per_node() {
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
    let lifecycle_only_node = graph.node().build();
    let lifecycle_and_output_node = graph.node().build();
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(lifecycle_only_observation_resource_declaration(
            lifecycle_only_node,
        ))
        .expect("lifecycle-only declaration should lower");
    runtime
        .declare_resource_node(resource_declaration(lifecycle_and_output_node))
        .expect("lifecycle-and-output declaration should lower");

    let lifecycle_only_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            lifecycle_only_node,
        )))
        .expect("lifecycle-only request should admit")
        .admitted_request();
    let lifecycle_and_output_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(
            lifecycle_and_output_node,
        )))
        .expect("lifecycle-and-output request should admit")
        .admitted_request();
    let lifecycle_only_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            lifecycle_only_node,
            lifecycle_only_request.handle(),
            lifecycle_only_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("lifecycle-only completion should admit");
    let lifecycle_and_output_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            lifecycle_and_output_node,
            lifecycle_and_output_request.handle(),
            lifecycle_and_output_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("lifecycle-and-output completion should admit");

    let calls = Arc::new(Mutex::new(Vec::<ResourceObservationRecord>::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [lifecycle_only_node, lifecycle_and_output_node],
        Box::new(ResourceObservationListener {
            calls: Arc::clone(&calls),
        }),
    );

    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let first = tx.stage_admitted_resource_completion(lifecycle_only_completion)?;
            let second = tx.stage_admitted_resource_completion(lifecycle_and_output_completion)?;
            tx.commit_staged_resource_completion(first.staged_effect())?;
            tx.commit_staged_resource_completion(second.staged_effect())?;
            Ok(())
        })
        .expect("two completions should commit in one transaction");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("committed resource observation should materialize");
    let event = &report.events()[0];

    assert_eq!(report.events().len(), 1);
    assert_eq!(event.matched_resource_nodes().len(), 2);
    assert_eq!(event.outcome(), ObservationBoundaryOutcome::Delivered);
    assert_eq!(
        event.matched_resource_nodes()[0].node(),
        ResourceNodeId::from_node(lifecycle_only_node)
    );
    assert_eq!(
        event.matched_resource_nodes()[0].lifecycle(),
        ResourceLifecycleClass::Fulfilled
    );
    assert_eq!(event.matched_resource_nodes()[0].output_continuity(), None);
    assert_eq!(
        event.matched_resource_nodes()[1].node(),
        ResourceNodeId::from_node(lifecycle_and_output_node)
    );
    assert_eq!(
        event.matched_resource_nodes()[1].output_continuity(),
        Some(ResourceOutputContinuity::OutputReplaced)
    );
    assert_eq!(
        report.performance().boundary(),
        ResourceBoundaryKind::ObservationMaterialization
    );
    assert_eq!(report.performance().input_width(), 1);
    assert_eq!(report.performance().admitted_count(), 1);
    assert_eq!(report.performance().coalescing_width(), 1);
    assert_eq!(
        report
            .performance()
            .output_continuity_classification_width(),
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_observation_policy_decision_count,
        2
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_observation_candidate_width,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_observation_coalesced_width,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_observation_delivered_width,
        1
    );
}

#[test]
fn resource_observation_batch_report_remains_rollback_safe() {
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
        .declare_resource_node(resource_declaration(node))
        .expect("resource declaration should lower");
    let admitted_request = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("request should admit")
        .admitted_request();
    let admitted_completion = runtime
        .admit_resource_completion(raw_completion(
            &runtime,
            node,
            admitted_request.handle(),
            admitted_request.attempt(),
            64,
        ))
        .admitted_completion()
        .expect("matching completion should admit");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    let staging = tx
        .stage_admitted_resource_completion(admitted_completion)
        .expect("completion should stage");
    tx.commit_staged_resource_completion(staging.staged_effect())
        .expect("completion should mutate transaction-local state");
    tx.rollback()
        .expect("rollback should restore resource state");

    let report = runtime
        .latest_resource_observation_batch_report()
        .expect("rollback-suppressed resource observation should materialize");
    let event = &report.events()[0];

    assert_eq!(
        event.outcome(),
        ObservationBoundaryOutcome::RollbackSuppressed
    );
    assert_eq!(event.matched_resource_nodes().len(), 1);
    assert_eq!(
        event.matched_resource_nodes()[0].lifecycle(),
        ResourceLifecycleClass::Pending
    );
    assert_eq!(
        event.matched_resource_nodes()[0].output_continuity(),
        Some(ResourceOutputContinuity::NoPriorOutput)
    );
    assert_eq!(report.performance().admitted_count(), 0);
    assert_eq!(report.performance().denied_count(), 1);
}
