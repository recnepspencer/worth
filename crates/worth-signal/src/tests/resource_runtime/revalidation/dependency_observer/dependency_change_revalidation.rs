use super::*;

#[test]
fn resource_dependency_change_revalidation_revalidates_invalidated_node_when_policy_allows() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(dependency_change_revalidation_resource_declaration(node))
        .expect("dependency-change declaration should lower");
    mark_dirty(runtime.graph_mut(), source, Aspect::new(0))
        .expect("dependency invalidation should mark resource node non-clean");
    assert!(matches!(
        runtime
            .graph()
            .get_state(node)
            .expect("resource node should exist"),
        NodeState::Dirty | NodeState::MaybeStale
    ));

    let proof = runtime
        .prove_dependency_change_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("invalidated node should mint dependency-change proof");
    let report = runtime
        .revalidate_resource_node_for_dependency_change(proof.clone())
        .expect("dependency-change proof should admit revalidation");
    let revalidation = report
        .admitted_revalidation()
        .expect("dependency-change proof should admit");
    let descriptor = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("descriptor should remain visible");

    assert_eq!(revalidation.expected_active(), None);
    assert_eq!(revalidation.forced_active_handle(), None);
    assert_eq!(
        revalidation
            .dependency_change_proof()
            .expect("admitted dependency-change revalidation should retain proof")
            .node_state(),
        proof.node_state()
    );
    assert_eq!(
        revalidation.decision_digest().as_str(),
        descriptor
            .revalidation_decision_plan()
            .decision_digest()
            .as_str()
    );
    assert_eq!(report.performance().temporal_wake_footprint(), 0);
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_dependency_change_revalidation_count,
        1
    );
}

#[test]
fn resource_dependency_change_revalidation_denies_forged_state_mismatch_proof() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(dependency_change_revalidation_resource_declaration(node))
        .expect("dependency-change declaration should lower");
    mark_dirty(runtime.graph_mut(), source, Aspect::new(0))
        .expect("dependency invalidation should mark resource node non-clean");
    let decision_digest = runtime
        .resource_descriptor_for_node(ResourceNodeId::from_node(node))
        .expect("descriptor should exist")
        .revalidation_decision_plan()
        .decision_digest()
        .clone();
    let forged = DependencyChangeResourceRevalidationProof::new(
        ResourceNodeId::from_node(node),
        NodeState::Clean,
        decision_digest,
    );

    let report = runtime
        .revalidate_resource_node_for_dependency_change(forged)
        .expect("forged proof denial should still be report-shaped");
    let denied = report
        .denied_revalidation()
        .expect("forged dependency-change proof must deny");

    assert_eq!(
        denied.class(),
        ResourceRevalidationDenialClass::DependencyChangeProofMismatch
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_revalidation_dependency_change_proof_mismatch_denial_count,
        1
    );
}

#[test]
fn resource_dependency_change_revalidation_does_not_bypass_active_request_rule() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(dependency_change_revalidation_resource_declaration(node))
        .expect("dependency-change declaration should lower");
    let active = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("initial request should admit")
        .admitted_request()
        .handle();
    mark_dirty(runtime.graph_mut(), source, Aspect::new(0))
        .expect("dependency invalidation should mark resource node non-clean");
    let proof = runtime
        .prove_dependency_change_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("invalidated active node should still mint invalidation proof");

    let report = runtime
        .revalidate_resource_node_for_dependency_change(proof)
        .expect("active-request denial should still be report-shaped");
    let denied = report
        .denied_revalidation()
        .expect("dependency-change proof must not bypass active overwrite rules");

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

#[test]
fn resource_observer_demand_and_dependency_change_revalidation_do_not_coalesce_across_distinct_freshness_causes(
) {
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
    let source = graph.node().build();
    let node = graph.node().build();
    graph
        .depends_on(node, source, Aspect::new(0))
        .expect("dependency edge should admit");
    let mut runtime = TestRuntime::build(graph);
    runtime
        .declare_resource_node(
            dependency_change_observer_demand_revalidation_resource_declaration(node),
        )
        .expect("combined revalidation declaration should lower");
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

    mark_dirty(runtime.graph_mut(), node, Aspect::new(0))
        .expect("dependency invalidation should mark node dirty");
    let dependency_proof = runtime
        .prove_dependency_change_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("dirty node should mint dependency-change proof");
    let dependency_report = runtime
        .revalidate_resource_node_for_dependency_change(dependency_proof)
        .expect("dependency-change proof should admit");
    let dependency_revalidation = dependency_report
        .admitted_revalidation()
        .expect("dependency-change proof should admit");
    assert_eq!(
        dependency_revalidation.freshness_decision().class(),
        ResourceRevalidationFreshnessClass::DependencyChange
    );

    let observer_proof = runtime
        .prove_observer_demand_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("committed observation should still mint observer-demand proof");
    let report = runtime
        .revalidate_resource_node_for_observer_demand(observer_proof)
        .expect("distinct freshness race should still be report-shaped");
    let denied = report
        .denied_revalidation()
        .expect("distinct freshness cause must not silently coalesce");

    assert_eq!(
        denied.class(),
        ResourceRevalidationDenialClass::ActiveRequestRequiresExpectedHandle
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_revalidation_coalesced_count,
        0
    );
}

#[test]
fn resource_observer_demand_revalidation_revalidates_using_committed_observation_truth() {
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
    let calls = Arc::new(Mutex::new(Vec::<ResourceObservationRecord>::new()));
    let observation_handle = runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [node],
        Box::new(ResourceObservationListener {
            calls: Arc::clone(&calls),
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
    let observation = tx.commit().expect("commit should succeed").observation;
    let delivered = calls
        .lock()
        .expect("resource observation mutex poisoned")
        .clone();
    assert_eq!(observation.delivered_event_count, 1);
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        delivered[0].observer_id,
        observation_handle.observer_id().get()
    );

    let proof = runtime
        .prove_observer_demand_resource_revalidation(ResourceNodeId::from_node(node))
        .expect("committed observation should mint observer-demand proof");
    let report = runtime
        .revalidate_resource_node_for_observer_demand(proof.clone())
        .expect("observer-demand proof should admit revalidation");
    let revalidation = report
        .admitted_revalidation()
        .expect("observer-demand proof should admit");

    assert_eq!(revalidation.expected_active(), None);
    assert_eq!(revalidation.forced_active_handle(), None);
    assert_eq!(
        revalidation
            .observer_demand_proof()
            .expect("admitted observer-demand revalidation should retain proof")
            .observation_digest(),
        proof.observation_digest()
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_observer_demand_revalidation_count,
        1
    );
    assert_eq!(
        runtime
            .telemetry()
            .resource
            .resource_revalidation_observer_demand_proof_check_count,
        1
    );
}
