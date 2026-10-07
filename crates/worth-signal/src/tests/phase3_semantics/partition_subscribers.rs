use crate::facade::{
    mark_dirty, mark_dirty_with_regions, ChangedRegion, NodeEvaluationResult, NodeId, NodeState,
    SignalGraph, UpstreamCause,
};
use crate::tests::support::{
    evaluate, version_ab, DependencyBatchBuilder, GraphDependencyBatchExt, ASPECT_A, ASPECT_B,
};

#[test]
fn partition_subscribers_only_dirty_on_matching_partition() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let wing_subscriber = graph.node().build();
    let tail_subscriber = graph.node().build();
    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing")))
    };
    evaluate(&mut graph, source, &mut source_v1).unwrap();

    graph
        .append_partition_dependency(wing_subscriber, source, ASPECT_A, "wing")
        .unwrap();
    graph
        .append_partition_dependency(tail_subscriber, source, ASPECT_A, "tail")
        .unwrap();

    let mut source_v2 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("wing")))
    };
    let mut subscriber_compute = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(10, 0)))
    };

    evaluate(&mut graph, wing_subscriber, &mut subscriber_compute).unwrap();
    evaluate(&mut graph, tail_subscriber, &mut subscriber_compute).unwrap();

    mark_dirty_with_regions(&mut graph, source, ASPECT_A, &[ChangedRegion::new("wing")]).unwrap();

    assert_eq!(graph.get_state(wing_subscriber).unwrap(), NodeState::Clean);
    assert_eq!(graph.get_state(tail_subscriber).unwrap(), NodeState::Clean);

    evaluate(&mut graph, source, &mut source_v2).unwrap();
    assert_eq!(graph.get_state(wing_subscriber).unwrap(), NodeState::Dirty);
    assert_eq!(graph.get_state(tail_subscriber).unwrap(), NodeState::Clean);
    let causes = graph.pending_causes(wing_subscriber).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].key.producer, source);
    assert_eq!(causes[0].key.aspect, ASPECT_A);
    assert_eq!(causes[0].changed_scopes.len(), 1);
    assert!(causes[0].changed_scopes.iter().any(|scope| scope
        .path()
        .segments()
        .first()
        .is_some_and(|part| part == "wing")));
}

#[test]
fn detail_sensitive_partition_subscriber_reverts_clean_when_detail_does_not_match() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let subscriber = graph.node().build();
    graph
        .append_partition_detail_dependency(subscriber, source, ASPECT_A, "wing", "rib-12")
        .unwrap();

    let mut source_rib_12 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")))
    };
    let mut source_rib_13 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")))
    };
    let mut subscriber_compute = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(10, 0)))
    };

    evaluate(&mut graph, source, &mut source_rib_12).unwrap();
    evaluate(&mut graph, subscriber, &mut subscriber_compute).unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();
    assert_eq!(graph.get_state(subscriber).unwrap(), NodeState::Clean);

    evaluate(&mut graph, source, &mut source_rib_13).unwrap();
    evaluate(&mut graph, subscriber, &mut subscriber_compute).unwrap();

    assert_eq!(graph.get_state(subscriber).unwrap(), NodeState::Clean);
    let explanation = graph.observe().explain(subscriber).unwrap();
    assert!(matches!(
        explanation.upstream.as_slice(),
        [UpstreamCause::Clean { subscription: Some(subscription), .. }]
        if subscription.path().segments() == ["wing", "rib-12"]
    ));
    assert_eq!(
        graph
            .observe()
            .metrics()
            .invalidation
            .partition_scope_revert_clean_count,
        0
    );
}

#[test]
fn mixed_whole_aspect_and_partition_subscribers_behave_deterministically() {
    let mut graph = SignalGraph::new();
    let source = graph.node().partitioned_output().build();
    let whole_aspect_subscriber = graph.node().build();
    let matching_partition_subscriber = graph.node().build();
    let non_matching_partition_subscriber = graph.node().build();
    let mut source_v1 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing")))
    };
    evaluate(&mut graph, source, &mut source_v1).unwrap();

    let mut dependencies = DependencyBatchBuilder::new(&mut graph);
    dependencies
        .append_dependency(whole_aspect_subscriber, source, ASPECT_A)
        .unwrap()
        .append_partition_dependency(matching_partition_subscriber, source, ASPECT_A, "wing")
        .unwrap()
        .append_partition_dependency(non_matching_partition_subscriber, source, ASPECT_A, "tail")
        .unwrap();
    dependencies.commit().unwrap();

    let mut source_v2 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("wing")))
    };
    let mut subscriber_compute = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(10, 0)))
    };

    evaluate(&mut graph, whole_aspect_subscriber, &mut subscriber_compute).unwrap();
    evaluate(
        &mut graph,
        matching_partition_subscriber,
        &mut subscriber_compute,
    )
    .unwrap();
    evaluate(
        &mut graph,
        non_matching_partition_subscriber,
        &mut subscriber_compute,
    )
    .unwrap();

    mark_dirty_with_regions(&mut graph, source, ASPECT_A, &[ChangedRegion::new("wing")]).unwrap();

    assert_eq!(
        graph.get_state(whole_aspect_subscriber).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        graph.get_state(matching_partition_subscriber).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        graph.get_state(non_matching_partition_subscriber).unwrap(),
        NodeState::Clean
    );

    evaluate(&mut graph, source, &mut source_v2).unwrap();
    assert_eq!(
        graph.get_state(whole_aspect_subscriber).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        graph.get_state(matching_partition_subscriber).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        graph.get_state(non_matching_partition_subscriber).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn partition_scoped_cleanup_does_not_hide_other_dirty_upstreams() {
    let mut graph = SignalGraph::new();
    let source_partitioned = graph.node().partitioned_output().build();
    let source_other = graph.node().build();
    let dependent = graph.node().build();
    graph
        .append_partition_detail_dependency(
            dependent,
            source_partitioned,
            ASPECT_A,
            "wing",
            "rib-12",
        )
        .unwrap();
    graph
        .append_dependency(dependent, source_other, ASPECT_B)
        .unwrap();

    let mut partitioned_v1 = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(1, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-12")))
    };
    let mut partitioned_v2_other_detail = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(2, 0))
            .with_changed_region(ChangedRegion::new("wing").with_detail("rib-13")))
    };
    let mut other_v1 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 1));
    let mut other_v2 = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(0, 2));
    let mut dependent_compute = |_id: NodeId, _graph: &SignalGraph| {
        Ok(NodeEvaluationResult::from_version(version_ab(10, 10)))
    };

    evaluate(&mut graph, source_partitioned, &mut partitioned_v1).unwrap();
    evaluate(&mut graph, source_other, &mut other_v1).unwrap();
    evaluate(&mut graph, dependent, &mut dependent_compute).unwrap();

    mark_dirty_with_regions(
        &mut graph,
        source_partitioned,
        ASPECT_A,
        &[ChangedRegion::new("wing").with_detail("rib-13")],
    )
    .unwrap();
    mark_dirty(&mut graph, source_other, ASPECT_B).unwrap();
    evaluate(
        &mut graph,
        source_partitioned,
        &mut partitioned_v2_other_detail,
    )
    .unwrap();
    assert_eq!(graph.get_state(dependent).unwrap(), NodeState::Clean);
    evaluate(&mut graph, source_other, &mut other_v2).unwrap();

    assert_ne!(graph.get_state(dependent).unwrap(), NodeState::Clean);
}
