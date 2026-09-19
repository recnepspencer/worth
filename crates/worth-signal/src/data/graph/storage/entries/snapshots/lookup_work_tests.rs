use super::*;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::logic::evaluation::EvaluationWork;

fn fixture() -> (SignalGraph, NodeId) {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let consumer = graph.create_node();
    graph
        .set_dep_snapshot(
            consumer,
            DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
                source,
                aspect: Aspect::new(0),
                cached_version: 7,
                scope: None,
            }]),
        )
        .unwrap();
    (graph, consumer)
}

#[test]
fn conditional_shape_lookup_uses_exact_shared_work_on_retained_storage() {
    let (mut graph, consumer) = fixture();
    let mut partition =
        crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition::retain_basis_storage(&mut graph);
    partition
        .execute(&mut graph, |selected| {
            let (_, id) = selected.node_dependency_ids(consumer).unwrap();
            let expected = selected.dependency_snapshot_shape_handle(id);
            let mut measured = Work::new(100_000);
            assert_eq!(
                selected
                    .dependency_snapshot_shape_handle_for_evaluation(
                        id,
                        &mut EvaluationWork::Conditional(&mut measured)
                    )
                    .unwrap(),
                expected
            );
            let cost = measured.visits();
            for available in [cost - 1, cost] {
                let mut work = Work::new(cost + 7);
                work.reserve_visits(cost + 7 - available).unwrap();
                let result = selected.dependency_snapshot_shape_handle_for_evaluation(
                    id,
                    &mut EvaluationWork::Conditional(&mut work),
                );
                if available == cost {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    assert_eq!(
                        result,
                        Err(SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: cost + 7
                        })
                    );
                }
            }
        })
        .unwrap();
}

#[test]
fn conditional_shape_lookup_denies_missing_indexes_without_reconstructing() {
    for missing_shapes in [false, true] {
        let (mut graph, consumer) = fixture();
        let (_, id) = graph.node_dependency_ids(consumer).unwrap();
        let expected = graph
            .dependency_snapshot_shape_handle_for_evaluation(
                id,
                &mut EvaluationWork::Conditional(&mut Work::new(100_000)),
            )
            .unwrap();
        // Native serde reconstitution deliberately omits derived indexes.
        if missing_shapes {
            graph.topology.dependency_snapshot_shapes = serde_json::from_str(
                &serde_json::to_string(&graph.topology.dependency_snapshot_shapes).unwrap(),
            )
            .unwrap();
        } else {
            graph.topology.dependency_snapshots = serde_json::from_str(
                &serde_json::to_string(&graph.topology.dependency_snapshots).unwrap(),
            )
            .unwrap();
        }
        let before = graph
            .topology
            .dependency_snapshots
            .require_retained_indexes(&graph.topology.dependency_snapshot_shapes);
        assert!(before.is_err());
        let result = graph.dependency_snapshot_shape_handle_for_evaluation(
            id,
            &mut EvaluationWork::Conditional(&mut Work::new(100_000)),
        );
        assert_eq!(result, Err(SignalError::SnapshotIndexUnavailable));
        assert_eq!(
            graph
                .topology
                .dependency_snapshots
                .require_retained_indexes(&graph.topology.dependency_snapshot_shapes),
            before
        );
        assert_ne!(
            expected,
            crate::data::dependency::SnapshotShapeHandle::EMPTY
        );
    }
}
