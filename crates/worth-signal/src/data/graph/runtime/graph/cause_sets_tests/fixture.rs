use super::{
    evaluate, Aspect, AspectMask, AspectVersion, DefaultComparatorPolicyResolver, DependencyEdge,
    DependencySnapshot, NodeId, ProducedAspectDelta, SignalGraph,
};

pub(in crate::data::graph::runtime::graph) fn graph_with_edge(
) -> (SignalGraph, NodeId, NodeId, Aspect) {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    let aspect = Aspect::new(2);
    let mut baseline = |_id, _graph: &SignalGraph| Ok(AspectVersion::zero());
    evaluate(&mut graph, producer, &mut baseline).unwrap();
    evaluate(&mut graph, consumer, &mut baseline).unwrap();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(producer, aspect)])
        .unwrap();
    let mut snapshot = DependencySnapshot::empty();
    snapshot.record(producer, aspect, 0, None);
    graph.set_dep_snapshot(consumer, snapshot).unwrap();
    (graph, producer, consumer, aspect)
}

pub(in crate::data::graph::runtime::graph) fn publish_delta(
    graph: &mut SignalGraph,
    producer: NodeId,
    aspect: Aspect,
    previous: u64,
    committed: u64,
    _ordinal: u64,
) {
    let ordinal = graph.cause_sets.reserve_output_commit_ordinal();
    graph
        .apply_node_aspect_version(
            producer,
            AspectVersion::from_updates([(aspect, committed)]),
            &[],
        )
        .unwrap();
    let delta = ProducedAspectDelta::from_committed_result(
        producer,
        ordinal,
        AspectVersion::from_updates([(aspect, previous)]),
        AspectVersion::from_updates([(aspect, committed)]),
        AspectMask::from_aspect(aspect),
        &[],
        &[],
    )
    .unwrap();
    let prepared = graph
        .prepare_direct_output_causes(
            &delta,
            &mut DefaultComparatorPolicyResolver::default(),
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
        .unwrap();
    // This fixture establishes publication for cause-lifecycle assertions.
    // Budget denial is tested by the conditional/waiter admission suites.
    let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(usize::MAX);
    let projection = crate::data::graph::PendingRevalidationNodeProjection::capture(
        &graph,
        delta.producer,
        &mut work,
    )
    .unwrap();
    let prepared = graph
        .prepare_direct_cause_publication(prepared, projection, false, &mut work)
        .unwrap();
    graph.publish_direct_output_causes(prepared).unwrap();
    graph.cause_sets.publish_output_commit(delta);
}
