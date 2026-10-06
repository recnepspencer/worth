use crate::facade::*;
use crate::tests::support::*;

#[test]
fn kv61_add_delete_10k_flat_memory() {
    let mut graph = SignalGraph::with_gc_threshold(100);

    for _ in 0..10_000 {
        let node = graph.node().build();
        graph.unregister_node(node).unwrap();

        if graph.should_gc() {
            graph.run_gc_epoch();
        }
    }

    assert!(
        graph.arena_capacity() <= 200,
        "Arena capacity {} should stay bounded via slot reuse",
        graph.arena_capacity()
    );
    assert_eq!(graph.active_node_count(), 0);
}

#[test]
fn kv62_delete_mid_chain_no_panic() {
    let mut graph = SignalGraph::new();
    let param = graph.node().build();
    let middle = graph.node().build();
    let feature = graph.node().build();

    graph.append_dependency(middle, param, ASPECT_B).unwrap();
    graph.append_dependency(feature, middle, ASPECT_B).unwrap();

    let mut compute = |_id, _g: &SignalGraph| Ok(version_ab(1, 1));
    evaluate(&mut graph, param, &mut compute).unwrap();
    evaluate(&mut graph, middle, &mut compute).unwrap();
    evaluate(&mut graph, feature, &mut compute).unwrap();

    graph.unregister_node(middle).unwrap();

    mark_dirty(&mut graph, param, ASPECT_B).unwrap();

    let feature_state = graph.get_state(feature).unwrap();
    assert_eq!(
        feature_state,
        NodeState::MaybeStale,
        "feature should require structural revalidation after its dependency is retired"
    );
}

#[test]
fn unregister_severs_subscriptions() {
    let mut graph = SignalGraph::new();
    let upstream = graph.node().build();
    let node = graph.node().build();
    let downstream = graph.node().build();

    graph.append_dependency(node, upstream, ASPECT_B).unwrap();
    graph.append_dependency(downstream, node, ASPECT_B).unwrap();

    graph.unregister_node(node).unwrap();

    let upstream_subs = graph.subscribers_of(upstream).unwrap();
    assert!(
        upstream_subs.is_empty(),
        "Upstream should have no subscribers after middle node deleted"
    );

    let downstream_deps = graph.dependencies_of(downstream).unwrap();
    assert!(downstream_deps.is_empty());
    assert!(
        graph
            .runtime_dependencies_of(downstream)
            .unwrap()
            .is_empty(),
        "runtime cleanup should prune stale downstream dependencies after unregister"
    );
}

#[test]
fn storage_pressure_accumulates_compaction_debt_without_tombstones() {
    let mut graph = SignalGraph::with_gc_threshold(1_000);
    let root = graph.node().build();
    let leaf = graph.node().build();

    for round in 0..12 {
        let scope = PartitionSubscription::partition_and_detail("book", format!("detail-{round}"));
        let edge = graph.build_dependency_edge(root, ASPECT_A, Some(scope));
        graph.set_dependency_edges_sorted(leaf, &[edge]).unwrap();
    }

    assert_eq!(graph.tombstone_count(), 0);
    assert!(graph.gc_compaction_debt_for_test() > 0);
}

#[test]
fn gc_epochs_burn_debt_incrementally_and_rotate_compaction_families() {
    let mut graph = SignalGraph::with_gc_threshold(1_000);
    graph.set_gc_compaction_state_for_test(2, 0);

    graph.run_gc_epoch();

    assert_eq!(graph.gc_compaction_cursor_for_test(), 1);
    assert_eq!(graph.gc_compaction_debt_for_test(), 1);

    graph.run_gc_epoch();

    assert_eq!(graph.gc_compaction_cursor_for_test(), 2);
    assert_eq!(graph.gc_compaction_debt_for_test(), 0);
}

#[test]
fn prepare_for_observation_runs_only_bounded_maintenance_work() {
    let mut graph = SignalGraph::with_gc_threshold(1_000);
    graph.set_gc_compaction_state_for_test(2, 0);

    graph.prepare_for_observation();

    assert_eq!(graph.gc_compaction_cursor_for_test(), 1);
    assert_eq!(graph.gc_compaction_debt_for_test(), 1);
}

#[test]
fn graph_strategy_reflects_gc_pressure_and_observation_profile() {
    let mut graph = SignalGraph::with_gc_threshold(5);
    let retired = graph.node().build();
    graph.unregister_node(retired).unwrap();
    graph.reset_runtime_policy_to_tier(DiagnosticsTier::Development);

    let strategy = graph.observe().evaluation_strategy();

    assert_eq!(strategy.gc_pressure, GcPressure::CompactAfterEvaluation);
    assert_eq!(strategy.observation_level, ObservationLevel::Full);
}

#[test]
fn graph_strategy_keeps_maintenance_policy_for_large_graphs() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::operational());
    for _ in 0..1_000 {
        graph.node().build();
    }

    let strategy = graph.observe().evaluation_strategy();

    assert_eq!(strategy.gc_pressure, GcPressure::Deferred);
    assert_eq!(strategy.observation_level, ObservationLevel::Minimal);
}

#[test]
fn runtime_default_evaluation_applies_strategy_gc_maintenance() {
    let mut graph = SignalGraph::with_gc_threshold(1);
    let node = graph.node().build();
    let retired = graph.node().build();
    graph.unregister_node(retired).unwrap();

    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    runtime
        .read(node, &(), &|view| Ok(view.finish(version_ab(1, 0))))
        .unwrap();

    assert_eq!(runtime.graph().tombstone_count(), 0);
    assert_eq!(
        runtime.observe().graph().telemetry().storage.gc_epoch_count,
        1
    );
}

#[test]
fn gc_epoch_compacts_arena() {
    let mut graph = SignalGraph::with_gc_threshold(5);

    let mut nodes = Vec::new();
    for _ in 0..10 {
        nodes.push(graph.node().build());
    }

    for node in &nodes[..5] {
        graph.unregister_node(*node).unwrap();
    }

    assert_eq!(graph.tombstone_count(), 5);
    graph.run_gc_epoch();
    assert_eq!(graph.tombstone_count(), 0);
}

#[test]
fn vacated_slot_reuse_preserves_generation_safety() {
    let mut graph = SignalGraph::new();
    let original = graph.node().build();
    graph.unregister_node(original).unwrap();
    let reused = graph.node().build();

    assert_ne!(original.generation(), reused.generation());
    assert!(!graph.is_alive(original));
    assert!(graph.is_alive(reused));
}

#[test]
fn double_unregister_is_rejected_without_free_list_corruption() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    graph.unregister_node(node).unwrap();
    let capacity_before = graph.arena_capacity();

    assert!(graph.unregister_node(node).is_err());

    let reused = graph.node().build();
    assert_eq!(graph.arena_capacity(), capacity_before);
    assert_ne!(reused.generation(), node.generation());
}
