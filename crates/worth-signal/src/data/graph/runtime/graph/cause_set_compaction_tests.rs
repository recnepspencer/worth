use super::cause_sets_tests::{graph_with_edge, publish_delta};
use super::SignalGraph;
use crate::data::proof::invalidation::binding::OutputCommitOrdinal;

#[test]
fn ordinary_release_reuses_slots_without_remapping_surviving_consumers() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    let mut consumers = vec![consumer];
    for _ in 0..3 {
        let node = graph.create_node();
        crate::tests::support::evaluate(&mut graph, node, &mut |_, _| {
            Ok(crate::data::aspect::AspectVersion::zero())
        })
        .unwrap();
        graph
            .set_dependencies(
                node,
                [crate::data::dependency::DependencyEdge::new(
                    producer, aspect,
                )],
            )
            .unwrap();
        let mut snapshot = crate::data::dependency::DependencySnapshot::empty();
        snapshot.record(producer, aspect, 0, None);
        graph.set_dep_snapshot(node, snapshot).unwrap();
        consumers.push(node);
    }
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let original_ids = consumers
        .iter()
        .map(|&node| graph.pending_cause_set_id(node).unwrap())
        .collect::<Vec<_>>();
    for (&node, &released) in consumers[..3].iter().zip(&original_ids[..3]) {
        graph.release_pending_causes(node).unwrap();
        assert!(graph.cause_sets.get(released).is_err());
        assert_eq!(
            graph.pending_cause_set_id(consumers[3]).unwrap(),
            original_ids[3]
        );
    }
    assert_eq!(graph.cause_sets.allocated_slot_count(), 4);
    assert_eq!(graph.cause_sets.occupied_slot_count(), 1);
    assert_eq!(graph.cause_sets.last_compaction_slot_visits(), 0);
    graph.release_pending_causes(consumers[3]).unwrap();
    assert_eq!(graph.cause_sets.occupied_slot_count(), 0);
    assert_eq!(graph.cause_sets.allocated_slot_count(), 4);
    publish_delta(&mut graph, producer, aspect, 1, 2, 2);
    assert_eq!(graph.cause_sets.allocated_slot_count(), 4);
    assert_eq!(graph.cause_sets.occupied_slot_count(), 4);
    assert_eq!(graph.cause_sets.last_compaction_slot_visits(), 0);
    for (&node, &old_id) in consumers.iter().zip(&original_ids) {
        assert!(graph.cause_sets.get(old_id).is_err());
        assert_eq!(
            graph.pending_causes(node).unwrap()[0]
                .binding_axes
                .committed_version,
            2
        );
    }
}

#[test]
fn retiring_dirty_consumer_releases_cause_authority_before_checkpoint() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    publish_delta(&mut graph, producer, aspect, 0, 1, 1);
    let ordinal = graph.cause_sets.output_commit_ordinal_for_test();

    assert_eq!(graph.cause_sets.occupied_slot_count(), 1);
    assert!(graph
        .cause_sets
        .published_output_commit(OutputCommitOrdinal(ordinal))
        .is_some());

    graph.unregister_node(consumer).unwrap();

    assert_eq!(graph.cause_sets.occupied_slot_count(), 0);
    assert_eq!(graph.cause_sets.allocated_slot_count(), 1);
    assert_eq!(graph.cause_sets.last_compaction_slot_visits(), 0);
    assert!(graph
        .cause_sets
        .published_output_commit(OutputCommitOrdinal(ordinal))
        .is_none());
    let authority = graph.capture_checkpoint_authority();
    let restored = SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    assert_eq!(restored.cause_sets.allocated_slot_count(), 0);
    assert_eq!(graph.cause_sets.allocated_slot_count(), 1);
    graph.compact_cause_set_storage().unwrap();
    assert_eq!(graph.cause_sets.allocated_slot_count(), 0);
    assert_eq!(graph.cause_sets.last_compaction_slot_visits(), 1);
}
