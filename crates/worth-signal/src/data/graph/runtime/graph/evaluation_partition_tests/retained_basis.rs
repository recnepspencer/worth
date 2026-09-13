use super::super::cause_sets_tests::{graph_with_edge, publish_delta};
use crate::data::graph::signal_graph::RetainedTopologyIndexDenial;
use crate::data::graph::storage::execution_basis::SignalExecutionBasis;
use crate::data::graph::storage::execution_basis::SignalExecutionBasisChargeDenial;
use crate::data::retained_storage::{
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial,
};

#[test]
fn seed_admission_denies_snapshot_reconstruction_before_creating_an_execution_slot() {
    use crate::data::dependency::{
        DependencySnapshot, DependencySnapshotEntry, DependencySnapshotIndexDenial,
    };
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    graph
        .set_dep_snapshot(
            consumer,
            DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
                source: producer,
                aspect,
                cached_version: 0,
                scope: None,
            }]),
        )
        .unwrap();
    let admitted =
        SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
    assert!(admitted.retained_storage_charge().bytes() > 0);
    let encoded = serde_json::to_string(&graph.topology.dependency_snapshots).unwrap();
    graph.topology.dependency_snapshots = serde_json::from_str(&encoded).unwrap();
    assert!(matches!(
        SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)),
        Err(SignalExecutionBasisChargeDenial::TopologyIndexes(
            RetainedTopologyIndexDenial::Snapshots(
                DependencySnapshotIndexDenial::SnapshotInternerRequiresReconstruction
            )
        ))
    ));
    // Prior admitted backing survives the failed new capture. Its retained
    // indexes are installed during activation instead of rebuilding current.
    let mut slot = admitted.new_evaluation_partition();
    slot.execute(&mut graph, |selected| {
        assert_eq!(
            selected.get_dep_snapshot(consumer).unwrap().entries()[0].source,
            producer
        );
    })
    .unwrap();
}

#[test]
fn seed_admission_rejects_unready_edge_indexes_without_repairing_them() {
    for expected in [
        RetainedTopologyIndexDenial::DependencyEdgesRequireReconstruction,
        RetainedTopologyIndexDenial::SubscriberEdgesRequireReconstruction,
        RetainedTopologyIndexDenial::ReverseSubscriptionsRequireReconstruction,
    ] {
        let (mut graph, producer, consumer, aspect) = graph_with_edge();
        let admitted =
            SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
        match expected {
            RetainedTopologyIndexDenial::DependencyEdgesRequireReconstruction => {
                let wire = serde_json::to_string(&graph.topology.dependency_edges).unwrap();
                graph.topology.dependency_edges = serde_json::from_str(&wire).unwrap();
            }
            RetainedTopologyIndexDenial::SubscriberEdgesRequireReconstruction => {
                let wire = serde_json::to_string(&graph.topology.subscriber_edges).unwrap();
                graph.topology.subscriber_edges = serde_json::from_str(&wire).unwrap();
            }
            RetainedTopologyIndexDenial::ReverseSubscriptionsRequireReconstruction => {
                graph.destroy_reverse_subscription_index_for_test();
            }
            RetainedTopologyIndexDenial::Snapshots(_) => unreachable!(),
        }
        assert_eq!(graph.topology.require_retained_indexes(), Err(expected));
        assert!(matches!(
            SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)),
            Err(SignalExecutionBasisChargeDenial::TopologyIndexes(actual)) if actual == expected
        ));
        assert_eq!(graph.topology.require_retained_indexes(), Err(expected));
        let mut slot = admitted.new_evaluation_partition();
        slot.execute(&mut graph, |selected| {
            assert_eq!(selected.topology.require_retained_indexes(), Ok(()));
            publish_delta(selected, producer, aspect, 0, 1, 1);
            assert_eq!(selected.pending_causes(consumer).unwrap().len(), 1);
        })
        .unwrap();
        assert_eq!(graph.topology.require_retained_indexes(), Err(expected));
    }
}

#[test]
fn empty_topology_seed_does_not_require_reconstruction() {
    let mut graph = crate::data::graph::SignalGraph::new();
    assert_eq!(graph.topology.require_retained_indexes(), Ok(()));
    SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
}

#[test]
fn captured_seed_carries_prepared_node_and_version_charges_into_each_slot() {
    let (mut graph, _, _, _) = graph_with_edge();
    let basis = SignalExecutionBasis::capture(&mut graph, &mut Preparation::new(100_000)).unwrap();
    for _ in 0..2 {
        let mut slot = basis.new_evaluation_partition();
        slot.execute(&mut graph, |selected| {
            // These lookups accept no work allowance and cannot repair by scan.
            assert!(selected.arena.hot.prepared_retained_charge().is_ok());
            assert!(selected.arena.warm.prepared_retained_charge().is_ok());
            assert!(selected.arena.cold.prepared_retained_charge().is_ok());
            assert!(selected
                .conditional_dependency_versions
                .prepared_retained_charge()
                .is_ok());
            assert!(selected
                .pending_repeated_invalidation_admissions
                .prepared_retained_charge()
                .is_ok());
        })
        .unwrap();
    }
}

#[test]
fn immutable_seed_charge_is_prepared_once_and_denies_incomplete_preparation() {
    let (mut graph, producer, _consumer, aspect) = graph_with_edge();
    let mut work = Preparation::new(100_000);
    let backing = SignalExecutionBasis::capture(&mut graph, &mut work).unwrap();
    let charge = backing.retained_storage_charge();
    assert!(charge.bytes() > std::mem::size_of::<SignalExecutionBasis>() as u64);
    // Independent native sources preserve the same cold preparation posture.
    // Reusing graph here would let its warmed charge facts hide missing work.
    let (mut exact_graph, _, _, _) = graph_with_edge();
    let exact =
        SignalExecutionBasis::capture(&mut exact_graph, &mut Preparation::new(work.visits()))
            .unwrap();
    assert_eq!(exact.retained_storage_charge(), charge);
    let (mut short_graph, _, _, _) = graph_with_edge();
    assert!(matches!(
        SignalExecutionBasis::capture(&mut short_graph, &mut Preparation::new(work.visits() - 1)),
        Err(SignalExecutionBasisChargeDenial::Storage(
            RetainedStoragePreparationDenial::WorkExhausted { .. }
        )) | Err(SignalExecutionBasisChargeDenial::Diagnostics(
            crate::diagnostics::state::BranchCarrierChargeDenial::Storage(
                RetainedStoragePreparationDenial::WorkExhausted { .. }
            )
        ))
    ));
    let mut slot = backing.new_evaluation_partition();
    slot.execute(&mut graph, |selected| {
        publish_delta(selected, producer, aspect, 0, 8, 1)
    })
    .unwrap();
    publish_delta(&mut graph, producer, aspect, 0, 9, 1);
    assert_eq!(backing.retained_storage_charge(), charge);
    drop(slot);
    assert_eq!(backing.retained_storage_charge(), charge);
}

#[test]
fn frozen_basis_seeds_isolated_slots_after_current_graph_and_first_slot_change() {
    let (mut graph, producer, consumer, aspect) = graph_with_edge();
    let backing = SignalExecutionBasis::capture(
        &mut graph,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(100_000),
    )
    .unwrap();
    let captured_pages = graph.arena.definitions.page_identities();
    let mut first = backing.new_evaluation_partition();
    first
        .execute(&mut graph, |selected| {
            publish_delta(selected, producer, aspect, 0, 4, 1);
            assert_eq!(selected.pending_causes(consumer).unwrap().len(), 1);
        })
        .unwrap();
    let added = graph.create_node();
    publish_delta(&mut graph, producer, aspect, 0, 9, 1);

    // Instantiation consumes only the immutable backing. Neither current graph
    // movement nor the first slot's derived work can contaminate this slot.
    let mut second = backing.new_evaluation_partition();
    drop(backing);
    second
        .execute(&mut graph, |selected| {
            assert!(selected.get_entry(added).is_err());
            assert_eq!(selected.arena.definitions.page_identities(), captured_pages);
            assert_eq!(
                selected
                    .node_version_for_scope(producer, aspect, None)
                    .unwrap(),
                0
            );
            assert!(selected.pending_causes(consumer).unwrap().is_empty());
            publish_delta(selected, producer, aspect, 0, 6, 1);
        })
        .unwrap();
    first
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_version_for_scope(producer, aspect, None)
                    .unwrap(),
                4
            );
            assert_eq!(selected.pending_causes(consumer).unwrap().len(), 1);
        })
        .unwrap();
    assert_eq!(
        graph
            .node_version_for_scope(producer, aspect, None)
            .unwrap(),
        9
    );
    assert!(graph.get_entry(added).is_ok());
}
