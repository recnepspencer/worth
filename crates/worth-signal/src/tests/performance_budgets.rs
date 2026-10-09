use crate::facade::*;
use crate::tests::support::*;

#[test]
fn active_node_count_stays_exact_through_sparse_churn() {
    let mut graph = SignalGraph::new();
    let nodes = (0..256).map(|_| graph.create_node()).collect::<Vec<_>>();

    for node in nodes.iter().step_by(3) {
        graph.unregister_node(*node).unwrap();
    }

    let expected = nodes.len() - nodes.iter().step_by(3).count();
    assert_eq!(graph.active_node_count(), expected);

    let replacements = (0..48).map(|_| graph.create_node()).collect::<Vec<_>>();
    assert_eq!(graph.active_node_count(), expected + replacements.len());
}

#[test]
fn compaction_reduces_segment_growth_after_repeated_snapshot_churn() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();

    for cycle in 0..64 {
        let mut snapshot = crate::data::dependency::DependencySnapshot::empty();
        snapshot.record(node, ASPECT_A, cycle as u64, None);
        graph.set_dep_snapshot(node, snapshot).unwrap();
    }

    let before = graph.storage_counts();
    graph.compact_graph_storage();
    let after = graph.storage_counts();

    assert!(after.0 .1 <= before.0 .1);
    assert!(after.1 .1 <= before.1 .1);
    assert!(after.2 <= before.2);
}

#[test]
fn overlapping_mark_dirty_calls_stage_only_the_changed_sources() {
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
    let a = graph.create_node();
    let b = graph.create_node();
    let c = graph.create_node();
    graph.append_dependency(b, a, ASPECT_A).unwrap();
    graph.append_dependency(c, b, ASPECT_A).unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let mut ctx = ();
    let mut tx = runtime.begin(request_execution, &mut ctx);
    tx.mark_dirty(a, ASPECT_A).unwrap();
    tx.mark_dirty(b, ASPECT_A).unwrap();
    tx.commit().unwrap();

    let metrics = runtime.observe().metrics();
    assert_eq!(metrics.transaction.max_touched_nodes_in_txn, 2);
    assert_eq!(
        metrics.transaction.transaction_mark_dirty_candidate_visits,
        2
    );
}

#[test]
fn repeated_gc_compaction_cycles_stay_near_live_edge_storage() {
    let mut runtime = SignalRuntime::builder(SignalGraph::with_gc_threshold(1))
        .with_kernel_defaults()
        .build();
    let root = runtime.graph_mut().node().build();
    let leaves = (0..24)
        .map(|_| runtime.graph_mut().node().build())
        .collect::<Vec<_>>();

    for round in 0..32 {
        for (index, &leaf) in leaves.iter().enumerate() {
            let aspect = if (round + index) % 2 == 0 {
                ASPECT_A
            } else {
                ASPECT_B
            };
            let _ = runtime.graph_mut().drop_dependency(leaf, root, ASPECT_A);
            let _ = runtime.graph_mut().drop_dependency(leaf, root, ASPECT_B);
            runtime
                .graph_mut()
                .append_dependency(leaf, root, aspect)
                .unwrap();
        }
        runtime.graph_mut().run_gc_epoch();
    }

    let ((dependency_edges, dependency_segments), (subscriber_edges, subscriber_segments), _) =
        runtime.graph().storage_counts();
    let live_nodes = runtime.graph().active_node_count();

    assert!(
        dependency_segments <= live_nodes * 2,
        "dependency segments should stay bounded near live-node scale after repeated compaction: edges={dependency_edges} segments={dependency_segments} live_nodes={live_nodes}"
    );
    assert!(
        subscriber_segments <= live_nodes * 2,
        "subscriber segments should stay bounded near live-node scale after repeated compaction: edges={subscriber_edges} segments={subscriber_segments} live_nodes={live_nodes}"
    );

    let metrics = runtime.graph().observe().metrics();
    assert!(metrics.storage.graph_storage_compaction_count >= 1);
    assert!(
        metrics.storage.graph_storage_dependency_segments_rewritten >= dependency_segments as u64
    );
    assert!(
        metrics.storage.graph_storage_subscriber_segments_rewritten >= subscriber_segments as u64
    );
}
