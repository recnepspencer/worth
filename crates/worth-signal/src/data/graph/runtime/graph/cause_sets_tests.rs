use super::SignalGraph;
use crate::data::aspect::{Aspect, AspectMask, AspectVersion};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{OutputCommitOrdinal, ResolvedDependencyCause};
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::proof::PartitionScopeSet;
use crate::tests::support::evaluate;

mod fixture;
pub(super) use fixture::{graph_with_edge, publish_delta};

#[test]
fn unscoped_cause_normalizes_to_whole_aspect_and_rebuilds_exact_cache() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);

    graph
        .get_entry_mut(consumer)
        .unwrap()
        .set_dirty_aspects(Default::default());
    graph
        .get_entry_mut(consumer)
        .unwrap()
        .clear_dirty_partition_scopes();
    graph
        .rebuild_dirty_caches_from_pending_causes(consumer)
        .unwrap();

    assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
    assert!(graph.pending_causes(consumer).unwrap()[0]
        .changed_scopes
        .is_empty());
    assert!(graph
        .node_dirty_aspects(consumer)
        .unwrap()
        .contains(crate::data::aspect::AspectMask::from_aspect(aspect)));
    assert!(graph
        .node_dirty_scoped_aspects(consumer)
        .unwrap()
        .is_empty());
}

#[test]
fn same_shaped_rewire_rejects_the_prior_dependency_revision() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let stale = graph.pending_causes(consumer).unwrap()[0].clone();
    graph.clear_dependencies(consumer).unwrap();
    graph
        .set_dependencies(consumer, [DependencyEdge::new(producer, aspect)])
        .unwrap();

    assert!(graph.replace_pending_causes(consumer, [stale]).is_err());
}

#[test]
fn cause_reversion_cannot_masquerade_as_a_direct_dirty_obligation() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 2, 1);
    assert_eq!(
        graph.get_state(consumer).unwrap(),
        crate::data::node::NodeState::Dirty
    );
    assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);

    publish_delta(&mut graph, producer, aspect, 2, 0, 2);
    assert_eq!(
        graph.get_state(consumer).unwrap(),
        crate::data::node::NodeState::MaybeStale
    );
    assert!(graph.pending_causes(consumer).unwrap().is_empty());
    assert!(graph.node_dirty_aspects(consumer).unwrap().is_empty());

    publish_delta(&mut graph, producer, aspect, 0, 3, 3);
    assert_eq!(
        graph.get_state(consumer).unwrap(),
        crate::data::node::NodeState::Dirty
    );
    assert_eq!(graph.pending_causes(consumer).unwrap().len(), 1);
    assert!(graph
        .node_dirty_aspects(consumer)
        .unwrap()
        .contains(AspectMask::from_aspect(aspect)));
}

#[test]
fn checkpoint_readmits_causes_and_compaction_rejects_stale_store_handles() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let old_id = graph.get_entry(consumer).unwrap().pending_cause_set_id();
    graph.compact_cause_set_storage().unwrap();
    assert!(graph.cause_sets.get(old_id).is_err());

    let snapshot_batch = graph.capture_checkpoint_dependency_snapshot_batch();
    let authority = graph.capture_checkpoint_authority();
    let mut restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    assert!(restored.pending_causes(consumer).is_err());
    let restore_batch = restored
        .derive_dependency_snapshot_restore_batch_from_checkpoint_batch(&authority, &snapshot_batch)
        .unwrap();
    restored
        .apply_classified_snapshot_batch_commit(restore_batch.classify())
        .unwrap();
    restored.readmit_checkpoint_causes().unwrap();
    let restored_consumer = restored.live_node_id_at(consumer.index() as usize).unwrap();
    let restored_cause = &restored.pending_causes(restored_consumer).unwrap()[0];
    assert_eq!(
        restored_cause.key.graph_instance,
        restored.runtime_instance_id()
    );
    assert!(restored_cause
        .binding()
        .ensure_matches(&restored_cause.binding())
        .is_ok());
}

#[test]
fn repeated_replacement_and_release_reuse_one_cause_slot_safely() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let first = graph.get_entry(consumer).unwrap().pending_cause_set_id();
    let mut previous = 1;
    for ordinal in 2..=100 {
        let committed = if previous == 1 { 2 } else { 1 };
        publish_delta(&mut graph, producer, aspect, previous, committed, ordinal);
        previous = committed;
    }
    assert_eq!(graph.cause_sets.allocated_slot_count(), 1);
    assert_eq!(graph.cause_sets.occupied_slot_count(), 1);

    graph.release_pending_causes(consumer).unwrap();
    assert!(graph.cause_sets.get(first).is_err());
    publish_delta(&mut graph, producer, aspect, previous, 3, 101);

    assert_eq!(graph.cause_sets.allocated_slot_count(), 1);
    assert_eq!(graph.cause_sets.occupied_slot_count(), 1);
    assert!(graph.cause_sets.get(first).is_err());
}

#[test]
fn checkpoint_payload_contains_only_reachable_cause_sets() {
    let (mut graph, producer, _consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let discarded = graph.node().build();
    graph
        .set_dependencies(discarded, [DependencyEdge::new(producer, aspect)])
        .unwrap();
    let mut discarded_snapshot = DependencySnapshot::empty();
    discarded_snapshot.record(producer, aspect, 1, None);
    graph
        .set_dep_snapshot(discarded, discarded_snapshot)
        .unwrap();
    publish_delta(&mut graph, producer, aspect, 1, 2, 2);
    graph.release_pending_causes(discarded).unwrap();

    let authority = graph.capture_checkpoint_authority();

    assert_eq!(authority.cause_sets.allocated_slot_count(), 1);
    assert_eq!(authority.cause_sets.occupied_slot_count(), 1);
}

#[test]
fn checkpoint_readmission_rejects_a_cause_bound_to_another_live_edge() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let unrelated = graph.node().build();
    graph
        .inject_pending_causes_unchecked_for_test(
            consumer,
            [ResolvedDependencyCause::new(
                graph.runtime_instance_id(),
                consumer,
                graph.dependency_revision(consumer).unwrap(),
                unrelated,
                aspect,
                None,
                1,
                OutputCommitOrdinal(1),
                1,
                PartitionScopeSet::default(),
            )],
        )
        .unwrap();
    let snapshot_batch = graph.capture_checkpoint_dependency_snapshot_batch();
    let authority = graph.capture_checkpoint_authority();
    let mut restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    let restore_batch = restored
        .derive_dependency_snapshot_restore_batch_from_checkpoint_batch(&authority, &snapshot_batch)
        .unwrap();
    restored
        .apply_classified_snapshot_batch_commit(restore_batch.classify())
        .unwrap();

    let error = restored
        .readmit_checkpoint_causes()
        .expect_err("restore must reject a cause not supported by current topology");

    assert!(error.to_string().contains("cause readmission failed"));
    assert!(restored.pending_causes(consumer).is_err());
    assert!(restored.is_alive(producer));
}

#[test]
fn restored_graph_serialization_is_stable_across_fresh_runtime_instances() {
    let (mut graph, producer, _consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };

    let left = SignalGraph::restore_from_checkpoint_image(&image).unwrap();
    let right = SignalGraph::restore_from_checkpoint_image(&image).unwrap();

    assert_ne!(left.runtime_instance_id(), right.runtime_instance_id());
    assert_eq!(
        serde_json::to_vec(&left).unwrap(),
        serde_json::to_vec(&right).unwrap()
    );
}

#[test]
fn restore_rejects_a_future_or_unassociated_output_commit_ordinal() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let mut tampered = graph.pending_causes(consumer).unwrap()[0].clone();
    tampered.binding_axes.output_commit_ordinal = OutputCommitOrdinal(2);
    graph
        .inject_pending_causes_unchecked_for_test(consumer, [tampered])
        .unwrap();
    let snapshot = graph.capture_snapshot();
    assert!(snapshot.authority_graph().is_err());
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };

    assert!(SignalGraph::restore_from_checkpoint_image(&image).is_err());
}

#[test]
fn restore_rejects_scope_payload_not_normalized_to_the_edge() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let mut tampered = graph.pending_causes(consumer).unwrap()[0].clone();
    tampered.changed_scopes =
        PartitionScopeSet::new(
            [crate::data::output::PartitionSubscription::whole_partition(
                "rates",
            )],
        );
    graph
        .inject_pending_causes_unchecked_for_test(consumer, [tampered])
        .unwrap();
    let image = crate::state::SignalCheckpointImage {
        authority: graph.capture_checkpoint_authority(),
        dependency_snapshot_batch: graph.capture_checkpoint_dependency_snapshot_batch(),
        graph_telemetry: *graph.telemetry(),
    };

    assert!(SignalGraph::restore_from_checkpoint_image(&image).is_err());
}

#[test]
fn raw_and_directly_deserialized_graphs_fail_closed_until_cause_readmission() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let authority = graph.capture_checkpoint_authority();
    let snapshot_batch = graph.capture_checkpoint_dependency_snapshot_batch();
    let mut raw = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    let mut compute = |_id, _graph: &SignalGraph| Ok(AspectVersion::zero());
    assert!(evaluate(&mut raw, consumer, &mut compute).is_err());

    let mut round_trip = graph.clone_stateful();
    round_trip.cause_sets =
        serde_json::from_slice(&serde_json::to_vec(&graph.cause_sets).unwrap()).unwrap();
    round_trip.cause_readmission_required = false;
    assert!(evaluate(&mut round_trip, consumer, &mut compute).is_err());
    round_trip.readmit_checkpoint_causes().unwrap();
    assert!(round_trip.pending_causes(consumer).is_ok());

    let restore_batch = raw
        .derive_dependency_snapshot_restore_batch_from_checkpoint_batch(&authority, &snapshot_batch)
        .unwrap();
    raw.apply_classified_snapshot_batch_commit(restore_batch.classify())
        .unwrap();
    raw.readmit_checkpoint_causes().unwrap();
    assert!(raw.pending_causes(consumer).is_ok());
}

#[test]
fn transient_wide_fanout_releases_references_and_retains_reusable_slots() {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let aspect = Aspect::new(2);
    let mut consumers = Vec::new();
    for _ in 0..64 {
        let consumer = graph.create_node();
        consumers.push(consumer);
    }
    let mut baseline = |_id, _graph: &SignalGraph| Ok(AspectVersion::zero());
    evaluate(&mut graph, producer, &mut baseline).unwrap();
    for &consumer in &consumers {
        evaluate(&mut graph, consumer, &mut baseline).unwrap();
        graph
            .set_dependencies(consumer, [DependencyEdge::new(producer, aspect)])
            .unwrap();
        let mut snapshot = DependencySnapshot::empty();
        snapshot.record(producer, aspect, 0, None);
        graph.set_dep_snapshot(consumer, snapshot).unwrap();
    }
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    assert_eq!(graph.cause_sets.occupied_slot_count(), consumers.len());
    let ordinal = graph.cause_sets.output_commit_ordinal_for_test();
    assert_eq!(
        graph
            .cause_sets
            .output_commit_reference_count_for_test(ordinal),
        consumers.len()
    );

    for (released, consumer) in consumers.into_iter().enumerate() {
        graph.release_pending_causes(consumer).unwrap();
        assert_eq!(
            graph
                .cause_sets
                .output_commit_reference_count_for_test(ordinal),
            63 - released
        );
    }
    assert_eq!(graph.cause_sets.occupied_slot_count(), 0);
    assert_eq!(graph.cause_sets.allocated_slot_count(), 64);
    assert!(graph
        .cause_sets
        .published_output_commit(OutputCommitOrdinal(ordinal))
        .is_none());
}
