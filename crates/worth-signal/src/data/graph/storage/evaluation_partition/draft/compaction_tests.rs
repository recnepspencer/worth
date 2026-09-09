use super::*;
use crate::data::dependency::{DependencySnapshot, DependencySnapshotEntry};
use crate::data::graph::storage::execution_basis::SignalExecutionBasis;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::retained_storage::RetainedStoragePreparation;
use crate::tests::support::ASPECT_A;

fn publish_snapshots(graph: &mut SignalGraph, producer: NodeId, consumer: NodeId) {
    // Native snapshot publication creates unique stored snapshots and invokes
    // storage-pressure policy. This is a draft custody test, not source proof.
    for cached_version in 1..=5 {
        graph
            .set_dep_snapshot(
                consumer,
                DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
                    source: producer,
                    aspect: ASPECT_A,
                    cached_version,
                    scope: None,
                }]),
            )
            .unwrap();
    }
    assert!(graph.arena.compaction.debt > 7);
    assert_eq!(graph.get_dep_snapshot(consumer).unwrap().entries().len(), 1);
}

#[test]
fn snapshot_pressure_follows_installed_rejected_and_unwound_evaluation_roots() {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    graph.arena.compaction.debt = 7;
    let basis =
        SignalExecutionBasis::capture(&mut graph, &mut RetainedStoragePreparation::new(100_000))
            .unwrap();
    // Ambient progress after capture must also survive every activation exit.
    graph.arena.compaction.debt = 11;
    for outcome in 0..3 {
        let mut partition = basis.new_evaluation_partition();
        if outcome == 2 {
            let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let draft = ConditionalEvaluationDraft::begin(
                    &mut partition,
                    &mut crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000),
                )
                .unwrap();
                draft
                    .partition
                    .execute(&mut graph, |selected| {
                        publish_snapshots(selected, producer, consumer);
                        panic!("unwind after native snapshot publication");
                    })
                    .unwrap();
            }));
            assert!(unwind.is_err());
        } else {
            let draft = ConditionalEvaluationDraft::begin(
                &mut partition,
                &mut crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000),
            )
            .unwrap();
            draft
                .partition
                .execute(&mut graph, |selected| {
                    publish_snapshots(selected, producer, consumer)
                })
                .unwrap();
            if outcome == 0 {
                drop(draft.reject());
            } else {
                draft.install();
            }
        }
        assert_eq!(graph.arena.compaction.debt, 11);
        assert!(graph
            .get_dep_snapshot(consumer)
            .unwrap()
            .entries()
            .is_empty());
        partition
            .execute(&mut graph, |selected| {
                if outcome == 1 {
                    assert!(selected.arena.compaction.debt > 7);
                    assert_eq!(
                        selected.get_dep_snapshot(consumer).unwrap().entries().len(),
                        1
                    );
                } else {
                    assert_eq!(selected.arena.compaction.debt, 7);
                    assert!(selected
                        .get_dep_snapshot(consumer)
                        .unwrap()
                        .entries()
                        .is_empty());
                }
            })
            .unwrap();
        basis
            .new_evaluation_partition()
            .execute(&mut graph, |sibling| {
                assert_eq!(sibling.arena.compaction.debt, 7);
                assert!(sibling
                    .get_dep_snapshot(consumer)
                    .unwrap()
                    .entries()
                    .is_empty());
            })
            .unwrap();
        assert_eq!(graph.arena.compaction.debt, 11);
    }
}
