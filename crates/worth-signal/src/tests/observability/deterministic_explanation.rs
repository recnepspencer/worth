use super::runtime_world::build_runtime;
use crate::facade::{
    mark_dirty, EvaluationRequestMode, NodeId, SignalError, SignalGraph, UpstreamCause,
};
use crate::tests::support::{
    evaluate, evaluate_on_demand, version_ab, DependencyBatchBuilder, GraphDependencyBatchExt,
    ASPECT_A, ASPECT_B,
};

#[test]
fn explanation_is_deterministic_with_multiple_upstreams_and_mixed_states() {
    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let source_c = graph.node().build();
    let dependent = graph.node().on_demand().build();
    let mut dependencies = DependencyBatchBuilder::new(&mut graph);
    dependencies
        .append_dependency(dependent, source_b, ASPECT_B)
        .unwrap()
        .append_dependency(dependent, source_a, ASPECT_A)
        .unwrap()
        .append_dependency(dependent, source_c, ASPECT_A)
        .unwrap();
    dependencies.commit().unwrap();

    let mut source_a_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut source_a_v2 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(2, 0));
    let mut source_b_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 1));
    let mut source_c_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(3, 0));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));

    evaluate(&mut graph, source_a, &mut source_a_v1).unwrap();
    evaluate(&mut graph, source_b, &mut source_b_v1).unwrap();
    evaluate(&mut graph, source_c, &mut source_c_v1).unwrap();
    evaluate_on_demand(&mut graph, dependent, &mut dependent_compute).unwrap();

    mark_dirty(&mut graph, source_a, ASPECT_A).unwrap();
    evaluate(&mut graph, source_a, &mut source_a_v2).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();

    let explanation = graph.observe().explain(dependent).unwrap();
    let rendered = format!("{explanation}");
    assert!(matches!(
        explanation.upstream.first(),
        Some(UpstreamCause::ConditionDeferred { source, .. }) if *source == source_a
    ));
    assert!(explanation.upstream.iter().any(|cause| matches!(
        cause,
        UpstreamCause::Clean { source, aspect, cached_version: 1, current_version: 1, .. }
        if *source == source_b && *aspect == ASPECT_B
    )));
    assert!(rendered.contains("condition OnDemand/Deferred"));
}

#[test]
fn rollback_preserves_committed_explanation_and_increments_rollback_metric() {
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
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let mut runtime = build_runtime(graph);

    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    let mut dependent_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(10, 0));
    evaluate(runtime.graph_mut(), source, &mut source_v1).unwrap();
    evaluate(runtime.graph_mut(), dependent, &mut dependent_v1).unwrap();
    let before = runtime.observe().explain(dependent).unwrap();
    let rollback_before = runtime
        .observe()
        .metrics()
        .transaction
        .transaction_rollback_count;

    let err = runtime.transaction(request_execution, &mut (), |tx| {
        tx.mark_dirty(source, ASPECT_A)?;
        tx.evaluate_with_plan(
            dependent,
            &|view| Ok(view.finish(version_ab(99, 0))),
            EvaluationRequestMode::Default,
        )?;
        Err(SignalError::invalid_input("rollback for test"))
    });
    assert!(err.is_err());

    let after = runtime.observe().explain(dependent).unwrap();
    assert_eq!(
        before.historical_artifact_record,
        after.historical_artifact_record
    );
    assert_eq!(before.upstream, after.upstream);
    assert_eq!(
        runtime
            .observe()
            .metrics()
            .transaction
            .transaction_rollback_count,
        rollback_before + 1
    );
}
