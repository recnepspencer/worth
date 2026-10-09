use super::runtime_world::{build_runtime, Ev};
use crate::facade::{CheckpointBarrier, NodeId, SignalGraph, TransactionOutcome};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_A, ASPECT_B};

#[test]
fn dependency_inspection_apis_are_deterministic() {
    let mut graph = SignalGraph::new();
    let root = graph.node().build();
    let middle = graph.node().build();
    let target = graph.node().build();
    graph.append_dependency(middle, root, ASPECT_A).unwrap();
    graph.append_dependency(target, middle, ASPECT_B).unwrap();

    assert_eq!(graph.dependencies_of(target).unwrap().len(), 1);
    assert_eq!(graph.subscribers_of(root).unwrap(), &[middle]);
    assert!(graph.depends_on(target, middle, ASPECT_B).unwrap());
    assert_eq!(
        graph.observe().dependency_chain_to(root, target).unwrap(),
        Some(vec![root, middle, target])
    );
}

#[test]
fn dot_export_contains_state_color_and_edge_labels() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().on_demand().build();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();

    let mut compute = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    evaluate(&mut graph, source, &mut compute).unwrap();

    let dot = graph.observe().to_dot();
    assert!(dot.contains(&format!("\"{}\"", source)));
    assert!(dot.contains("fillcolor=green"));
    assert!(dot.contains("aspect:0"));
    assert!(dot.contains("scope:"));
}

#[test]
fn metrics_snapshots_reflect_runtime_activity() {
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

    let outcome = runtime
        .transaction(request_execution, &mut (), |transaction| {
            transaction.mark_dirty(source, ASPECT_A)?;
            transaction.emit_event(Ev::Tick);
            transaction.flush_events(CheckpointBarrier::PerOperation)?;
            Ok(())
        })
        .unwrap();

    assert_eq!(outcome.outcome, TransactionOutcome::Committed);
    assert!(
        runtime
            .observe()
            .metrics()
            .transaction
            .transaction_begin_count
            >= 1
    );
    assert!(
        runtime
            .observe()
            .metrics()
            .transaction
            .transaction_commit_count
            >= 1
    );
    assert!(runtime.observe().metrics().checkpoint.event_flushes >= 1);
    let estimate = runtime
        .graph()
        .observe()
        .latest_invalidation_planning_estimate()
        .expect("committed source seed should retain a planning estimate");
    assert_eq!(estimate.seed_count(), 1);
    assert_eq!(estimate.direct_candidate_count(), 0);
}
