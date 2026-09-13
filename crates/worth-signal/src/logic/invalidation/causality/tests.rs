use crate::data::aspect::{Aspect, AspectMask, AspectVersion};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::data::graph::SignalGraph;
use crate::data::output::{ChangedRegion, PartitionSubscription};
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::proof::PartitionScopeSet;
use crate::tests::support::evaluate;

fn publish_scoped_delta(
    graph: &mut SignalGraph,
    producer: crate::data::handle::NodeId,
    aspect: Aspect,
    region: ChangedRegion,
) -> crate::data::proof::invalidation::binding::OutputCommitOrdinal {
    let ordinal = graph.cause_sets.reserve_output_commit_ordinal();
    let committed = AspectVersion::from_updates([(aspect, 1)]);
    graph
        .apply_node_aspect_version(producer, committed, &[])
        .unwrap();
    let delta = ProducedAspectDelta::from_committed_result(
        producer,
        ordinal,
        AspectVersion::zero(),
        committed,
        AspectMask::from_aspect(aspect),
        &[(aspect, region)],
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
    let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(10_000);
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
    ordinal
}

fn evaluated_graph_with_edge(
    edge: impl FnOnce(crate::data::handle::NodeId, Aspect) -> DependencyEdge,
) -> (
    SignalGraph,
    crate::data::handle::NodeId,
    crate::data::handle::NodeId,
    Aspect,
) {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    let aspect = Aspect::new(3);
    let edge = edge(producer, aspect);
    let mut baseline = |_id, _graph: &SignalGraph| Ok(AspectVersion::zero());
    evaluate(&mut graph, producer, &mut baseline).unwrap();
    evaluate(&mut graph, consumer, &mut baseline).unwrap();
    graph.set_dependencies(consumer, [edge.clone()]).unwrap();
    let mut snapshot = DependencySnapshot::empty();
    snapshot.record(producer, aspect, 0, edge.scope_ref().cloned());
    graph.set_dep_snapshot(consumer, snapshot).unwrap();
    (graph, producer, consumer, aspect)
}

#[test]
fn whole_partition_edge_normalizes_detail_change_to_one_whole_partition_scope() {
    let (mut graph, producer, consumer, aspect) = evaluated_graph_with_edge(|producer, aspect| {
        DependencyEdge::whole_partition(producer, aspect, "rates")
    });
    publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("rates").with_detail("5y"),
    );

    assert_eq!(
        graph.pending_causes(consumer).unwrap()[0]
            .changed_scopes
            .as_slice(),
        &[PartitionSubscription::whole_partition("rates")]
    );
}

#[test]
fn detail_edge_normalizes_change_to_that_exact_edge_scope() {
    let (mut graph, producer, consumer, aspect) = evaluated_graph_with_edge(|producer, aspect| {
        DependencyEdge::partition_detail(producer, aspect, "rates", "5y")
    });
    publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("rates").with_detail("5y"),
    );

    assert_eq!(
        graph.pending_causes(consumer).unwrap()[0]
            .changed_scopes
            .as_slice(),
        &[PartitionSubscription::partition_and_detail("rates", "5y")]
    );
}

#[test]
fn uninterned_changed_partition_still_publishes_to_unscoped_consumer() {
    let (mut graph, producer, consumer, aspect) = evaluated_graph_with_edge(DependencyEdge::new);
    let ordinal = publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("previously-unseen-partition").with_detail("previously-unseen-detail"),
    );

    let causes = graph.pending_causes(consumer).unwrap();
    assert_eq!(causes.len(), 1);
    assert_eq!(causes[0].key.producer, producer);
    assert_eq!(causes[0].key.aspect, aspect);
    assert_eq!(causes[0].binding_axes.output_commit_ordinal, ordinal);
}

#[test]
fn restore_rejects_commit_scope_that_no_longer_supports_the_bound_edge() {
    let (mut graph, producer, _consumer, aspect) = evaluated_graph_with_edge(|producer, aspect| {
        DependencyEdge::partition_detail(producer, aspect, "rates", "5y")
    });
    let ordinal = publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("rates").with_detail("5y"),
    );
    graph.cause_sets.replace_published_change_scopes_for_test(
        ordinal,
        PartitionScopeSet::new([PartitionSubscription::partition_and_detail("rates", "10y")]),
    );
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };

    assert!(SignalGraph::restore_from_checkpoint_image(&image).is_err());
}

#[test]
fn restore_rejects_delta_whose_internal_ordinal_contradicts_its_ledger_key() {
    let (mut graph, producer, _consumer, aspect) = evaluated_graph_with_edge(|producer, aspect| {
        DependencyEdge::partition_detail(producer, aspect, "rates", "5y")
    });
    let ordinal = publish_scoped_delta(
        &mut graph,
        producer,
        aspect,
        ChangedRegion::new("rates").with_detail("5y"),
    );
    graph
        .cause_sets
        .replace_published_internal_ordinal_for_test(
            ordinal,
            crate::data::proof::invalidation::binding::OutputCommitOrdinal(2),
        );
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };

    assert!(SignalGraph::restore_from_checkpoint_image(&image).is_err());
}

mod work;
