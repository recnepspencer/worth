use super::*;

#[test]
fn resource_observer_demand_revalidation_requires_committed_not_rollback_suppressed_observation() {
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
        .declare_resource_node(observer_demand_revalidation_resource_declaration(node))
        .expect("observer-demand declaration should lower");
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );

    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    let observation = tx.rollback().expect("rollback should succeed").observation;
    assert_eq!(observation.delivered_event_count, 0);
    assert_eq!(observation.rollback_suppressed_event_count, 1);

    let err = runtime
        .prove_observer_demand_resource_revalidation(ResourceNodeId::from_node(node))
        .expect_err("rollback-suppressed observation must not mint observer-demand proof");
    assert!(err
        .to_string()
        .contains("without committed matching observation"));
}

#[test]
fn resource_observer_demand_revalidation_denies_forged_observation_proof() {
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
        .declare_resource_node(observer_demand_revalidation_resource_declaration(node))
        .expect("observer-demand declaration should lower");
    let observation_handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");

    let decision_digest = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("descriptor should exist")
        .revalidation_decision_plan()
        .decision_digest()
        .clone();
    let forged = ObserverDemandResourceRevalidationProof::new(
        ResourceNodeId::from_node(node),
        observation_handle.observer_id().get(),
        observation_handle.handle_id().get() + 1,
        String::from("forged-observation-digest"),
        decision_digest,
    );

    let report = runtime
        .revalidate_resource_node_for_observer_demand(forged)
        .expect("forged observer-demand proof denial should still be report-shaped");
    let denied = report
        .denied_revalidation()
        .expect("forged observer-demand proof must deny");

    assert_eq!(
        denied.class(),
        ResourceRevalidationDenialClass::ObserverDemandProofMismatch
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_revalidation_observer_demand_proof_mismatch_denial_count,
        1
    );
}

#[test]
fn resource_observer_demand_revalidation_does_not_bypass_active_request_rule() {
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
        .declare_resource_node(observer_demand_revalidation_resource_declaration(node))
        .expect("observer-demand declaration should lower");
    let active = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request()
        .handle();
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::new(Mutex::new(Vec::new())),
        }),
    );
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.evaluate_with_plan(
        node,
        &|view| Ok(view.finish(NodeEvaluationResult::from_version(version_ab(1, 0)))),
        EvaluationRequestMode::Default,
    )
    .expect("evaluation should succeed");
    tx.commit().expect("commit should succeed");
    let proof = runtime
        .prove_observer_demand_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("committed observation should mint observer-demand proof");

    let report = runtime
        .revalidate_resource_node_for_observer_demand(proof)
        .expect("active-request denial should still be report-shaped");
    let denied = report
        .denied_revalidation()
        .expect("observer-demand proof must not bypass active overwrite rules");

    assert_eq!(
        denied.class(),
        ResourceRevalidationDenialClass::ActiveRequestRequiresExpectedHandle
    );
    assert_eq!(
        runtime
            .in_flight_resource_request(active)
            .expect("active request should remain authoritative")
            .status(),
        ResourceInFlightStatus::Active
    );
}
