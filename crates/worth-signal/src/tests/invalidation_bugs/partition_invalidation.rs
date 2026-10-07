use crate::facade::*;
use crate::tests::support::*;

#[test]
fn partition_scoped_dependencies_on_same_source_check_all_matching_edges() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let dependent = graph.node().build();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-13")
        .unwrap();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12"))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")))
    })
    .unwrap();
    evaluate(&mut graph, dependent, &mut |_id, graph| {
        let version = graph.get_entry(source).unwrap().get_aspect_version();
        Ok(NodeEvaluationResult::from_version(version))
    })
    .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();

    assert_eq!(
        graph.get_state(dependent).unwrap(),
        NodeState::Clean,
        "source dirtiness must not become subscriber authority before output commit"
    );

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")))
    })
    .unwrap();
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Dirty);
}

#[test]
fn repeated_partition_invalidations_union_dirty_scopes_until_evaluation() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let dependent = graph.node().build();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-13")
        .unwrap();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12"))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")))
    })
    .unwrap();
    evaluate(&mut graph, dependent, &mut |_id, graph| {
        let version = graph.get_entry(source).unwrap().get_aspect_version();
        Ok(NodeEvaluationResult::from_version(version))
    })
    .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-12")],
    )
    .unwrap();
    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();

    let scopes = graph.node_dirty_scoped_aspects(source).unwrap();
    assert_eq!(graph.get_state(source).unwrap(), NodeState::Dirty);
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);
    assert!(
        scopes
            .iter()
            .any(|(_, scope)| { scope.path().segments() == ["wing", "rib-12"] }),
        "the first invalidation scope should not be erased by a later wave"
    );
    assert!(
        scopes
            .iter()
            .any(|(_, scope)| { scope.path().segments() == ["wing", "rib-13"] }),
        "the second invalidation scope should be merged with earlier scopes"
    );
}

#[test]
fn whole_aspect_invalidation_does_not_erase_other_aspects_partition_precision() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let dependent = graph.node().build();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_A, "wing", "rib-12")
        .unwrap();
    graph
        .append_partition_detail_dependency(dependent, source, ASPECT_B, "tail", "panel-7")
        .unwrap();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 1))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12"))
            .with_changed_region(ChangedRegion::new("tail").with_detail("panel-7")))
    })
    .unwrap();
    evaluate(&mut graph, dependent, &mut |_id, graph| {
        Ok(NodeEvaluationResult::from_version(
            graph.get_entry(source).unwrap().get_aspect_version(),
        ))
    })
    .unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_B,
        &[ChangedRegion::new("tail").with_detail("panel-7")],
    )
    .unwrap();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();

    let scopes = graph.node_dirty_scoped_aspects(source).unwrap();
    assert!(
        scopes.iter().any(|(_, scope)| {
            scope.path().segments() == ["tail", "panel-7"]
        }),
        "whole-aspect invalidation on aspect A must not erase scoped dirtiness retained for aspect B"
    );
}

#[test]
fn root_invalidation_does_not_walk_a_reconverging_frontier() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let b = graph.node().build();
    let c = graph.node().build();
    let d = graph.node().build();
    let e = graph.node().build();

    graph.append_dependency(b, source, ASPECT_A).unwrap();
    graph.append_dependency(c, source, ASPECT_A).unwrap();
    graph.append_dependency(e, source, ASPECT_A).unwrap();
    graph.append_dependency(d, b, ASPECT_A).unwrap();
    graph.append_dependency(d, c, ASPECT_A).unwrap();
    graph.append_dependency(e, d, ASPECT_A).unwrap();

    evaluate(&mut graph, source, &mut |_id, _graph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0)))
    })
    .unwrap();
    for node in [b, c, d, e] {
        evaluate(&mut graph, node, &mut |_id, _graph| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0)))
        })
        .unwrap();
    }

    graph.reset_telemetry();
    mark_dirty(&mut graph, source, ASPECT_A).unwrap();

    assert_eq!(graph.telemetry().invalidation.invalidation_nodes_visited, 0);
    for node in [b, c, d, e] {
        assert_eq!(graph.get_state(node).unwrap(), NodeState::Clean);
    }
}

#[test]
#[ignore = "stress-scale invalidation depth; keep out of the default dev loop"]
fn deep_invalidation_chain_completes_without_recursive_cycle_detection() {
    let mut graph = SignalGraph::new();
    let root = graph.node().build();
    let mut previous = root;
    for _ in 0..20_000 {
        let next = graph.node().build();
        graph.append_dependency(next, previous, ASPECT_A).unwrap();
        previous = next;
    }

    let result = mark_dirty(&mut graph, root, ASPECT_A);
    assert!(
        result.is_ok(),
        "deep invalidation chains should not overflow recursive cycle detection: {result:?}"
    );
}

#[test]
fn batch_invalidation_does_not_open_a_transitive_wave() {
    let mut graph = SignalGraph::new();
    let source_a = graph.node().build();
    let source_b = graph.node().build();
    let shared = graph.node().build();
    let leaf = graph.node().build();

    graph.append_dependency(shared, source_a, ASPECT_A).unwrap();
    graph.append_dependency(shared, source_b, ASPECT_A).unwrap();
    graph.append_dependency(leaf, shared, ASPECT_A).unwrap();

    for node in [source_a, source_b, shared, leaf] {
        evaluate(&mut graph, node, &mut |_id, _graph| {
            Ok(NodeEvaluationResult::from_version(version_ab(1, 0)))
        })
        .unwrap();
    }

    let mut scalar = graph.clone();
    scalar.reset_telemetry();
    mark_dirty(&mut scalar, source_a, ASPECT_A).unwrap();
    mark_dirty(&mut scalar, source_b, ASPECT_A).unwrap();

    let mut batched = graph;
    batched.reset_telemetry();
    mark_dirty_batch(
        &mut batched,
        &DirtyBatch::from_sources([(source_a, ASPECT_A), (source_b, ASPECT_A)]),
    )
    .unwrap();

    assert_eq!(batched.get_state(shared).unwrap(), NodeState::Clean);
    assert_eq!(batched.get_state(leaf).unwrap(), NodeState::Clean);
    assert_eq!(
        batched.telemetry().invalidation.invalidation_nodes_visited,
        0
    );
    assert_eq!(
        scalar.telemetry().invalidation.invalidation_nodes_visited,
        0
    );
    assert_eq!(batched.telemetry().invalidation.frontier_seed_count, 2);
}
