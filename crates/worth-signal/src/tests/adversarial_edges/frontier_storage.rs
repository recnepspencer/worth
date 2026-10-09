use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::facade::{
    mark_dirty, EvaluationRequestMode, NodeEvaluationResult, SignalGraph, SignalRuntime,
};
use crate::tests::support::{
    version_ab, DependencyBatchBuilder, GraphDependencyBatchExt, ASPECT_A, ASPECT_B,
};

#[test]
fn reconverging_invalidation_path_is_not_reported_as_a_cycle() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let direct_b = graph.node().build();
    let direct_c = graph.node().build();
    let direct_d = graph.node().build();
    let direct_e = graph.node().build();

    let mut dependencies = DependencyBatchBuilder::new(&mut graph);
    dependencies
        .append_dependency(direct_b, source, ASPECT_A)
        .unwrap()
        .append_dependency(direct_c, source, ASPECT_A)
        .unwrap()
        .append_dependency(direct_d, source, ASPECT_A)
        .unwrap()
        .append_dependency(direct_e, source, ASPECT_A)
        .unwrap()
        .append_dependency(direct_d, direct_b, ASPECT_A)
        .unwrap()
        .append_dependency(direct_d, direct_c, ASPECT_A)
        .unwrap()
        .append_dependency(direct_e, direct_d, ASPECT_A)
        .unwrap();
    dependencies.commit().unwrap();

    let result = mark_dirty(&mut graph, source, ASPECT_A);

    assert!(
        result.is_ok(),
        "reconverging DAG invalidation should not be treated as a circular reference: {result:?}"
    );
}

#[test]
fn gc_epoch_compacts_edge_and_snapshot_storage_after_churn() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::with_gc_threshold(1))
        .with_kernel_defaults()
        .build();
    let source_a = runtime.graph_mut().node().build();
    let source_b = runtime.graph_mut().node().build();
    let dependent = runtime.graph_mut().node().build();
    let mut runtime_ctx = ();

    for round in 0..24 {
        runtime
            .graph_mut()
            .set_dependencies(
                dependent,
                [DependencyEdge::new(
                    if round % 2 == 0 { source_a } else { source_b },
                    ASPECT_A,
                )],
            )
            .unwrap();
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(if round % 2 == 0 { source_a } else { source_b }, ASPECT_A)?;
                tx.evaluate_with_plan(
                    dependent,
                    &|view| {
                        let result = if view.node() == source_a {
                            view.finish(version_ab(round as u64 + 1, 0))
                        } else if view.node() == source_b {
                            view.finish(version_ab(round as u64 + 100, 0))
                        } else {
                            let source = if round % 2 == 0 { source_a } else { source_b };
                            let version = view.read_aspect_version(source, ASPECT_A)?;
                            view.finish(NodeEvaluationResult::from_version(version))
                        };
                        Ok(result)
                    },
                    EvaluationRequestMode::Default,
                )?;
                Ok(())
            })
            .unwrap();
    }

    let before = runtime.graph().test_storage_counts();
    runtime.graph_mut().run_gc_epoch();
    let after = runtime.graph().test_storage_counts();

    assert!(
        after.0 .1 <= 4,
        "dependency edge segments should compact back near live-node cardinality after GC: before={before:?} after={after:?}"
    );
    assert!(
        after.1 .1 <= 4,
        "subscriber edge segments should compact back near live-node cardinality after GC: before={before:?} after={after:?}"
    );
    assert!(
        after.2 <= 2,
        "dependency snapshots should compact back near live snapshot count after GC: before={before:?} after={after:?}"
    );
}

#[test]
fn semantically_identical_dependency_snapshots_deduplicate_even_if_recorded_in_different_orders() {
    let mut graph = SignalGraph::new();
    let a = graph.node().build();
    let b = graph.node().build();
    let dependent = graph.node().build();

    let mut left = DependencySnapshot::empty();
    left.record(a, ASPECT_A, 1, None);
    left.record(b, ASPECT_B, 2, None);

    let mut right = DependencySnapshot::empty();
    right.record(b, ASPECT_B, 2, None);
    right.record(a, ASPECT_A, 1, None);

    graph.set_dep_snapshot(dependent, left).unwrap();
    let first = graph.get_entry(dependent).unwrap().get_dep_snapshot_id();
    graph.set_dep_snapshot(dependent, right).unwrap();
    let second = graph.get_entry(dependent).unwrap().get_dep_snapshot_id();

    assert_eq!(
        first, second,
        "snapshot storage should deduplicate canonical-equal snapshots regardless of record order"
    );
}

#[test]
fn dependency_snapshot_growth_returns_near_live_state_after_gc() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::with_gc_threshold(1))
        .with_kernel_defaults()
        .build();
    let source = runtime.graph_mut().node().build();
    let dependent = runtime.graph_mut().node().build();
    runtime
        .graph_mut()
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();
    let mut runtime_ctx = ();

    for round in 0..64 {
        runtime
            .transaction(request_execution, &mut runtime_ctx, |tx| {
                tx.mark_dirty(source, ASPECT_A)?;
                tx.evaluate_with_plan(
                    dependent,
                    &|view| {
                        let result = if view.node() == source {
                            view.finish(version_ab(round as u64 + 1, 0))
                        } else {
                            let version = view.read_aspect_version(source, ASPECT_A)?;
                            view.finish(NodeEvaluationResult::from_version(version))
                        };
                        Ok(result)
                    },
                    EvaluationRequestMode::Default,
                )?;
                Ok(())
            })
            .unwrap();
    }

    let before = runtime.graph().test_storage_counts();
    runtime.graph_mut().run_gc_epoch();
    let after = runtime.graph().test_storage_counts();
    assert!(
        after.2 <= 2,
        "dependency snapshot storage should compact back near live snapshot count after churn: before={before:?} after={after:?}"
    );
}

#[test]
fn identical_dependency_snapshots_are_deduplicated_before_gc() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_dependency(dependent, source, ASPECT_A)
        .unwrap();

    let mut snapshot = DependencySnapshot::empty();
    snapshot.record(source, ASPECT_A, 1, None);
    graph.set_dep_snapshot(dependent, snapshot.clone()).unwrap();
    let first = graph.get_entry(dependent).unwrap().get_dep_snapshot_id();
    graph.set_dep_snapshot(dependent, snapshot).unwrap();
    let second = graph.get_entry(dependent).unwrap().get_dep_snapshot_id();

    assert_eq!(first, second);
    assert_eq!(graph.test_storage_counts().2, 1);
}
