use super::*;
use crate::data::aspect::{Aspect, AspectVersion};
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

fn fixture(stable: bool, text: &str) -> (SignalGraph, NodeId) {
    let mut graph = SignalGraph::new();
    let producer = graph.create_node();
    let consumer = graph.create_node();
    graph
        .apply_node_aspect_version(
            producer,
            AspectVersion::from_updates([(Aspect::new(0), 9)]),
            &[],
        )
        .unwrap();
    let scope = PartitionSubscription::partition_and_detail(text, text);
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::partition_detail(
                producer,
                Aspect::new(0),
                text,
                text,
            )],
        )
        .unwrap();
    if stable {
        graph
            .set_dep_snapshot(
                consumer,
                DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
                    source: producer,
                    aspect: Aspect::new(0),
                    cached_version: 1,
                    scope: Some(scope),
                }]),
            )
            .unwrap();
    }
    (graph, consumer)
}

#[test]
fn dependency_capture_preserves_snapshot_and_uses_one_exact_or_short_allowance() {
    for stable in [false, true] {
        let (mut graph, node) = fixture(stable, "scope-λ");
        let before = graph.get_dep_snapshot(node).unwrap().clone();
        let mut measured = Work::new(1_000_000);
        let expected = resolve_effect_dependency_inputs(
            &mut graph,
            node,
            None,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
        assert_eq!(expected.dependency_snapshot_update.entry_count(), 1);
        assert_eq!(
            expected.dependency_snapshot_update.change_kind(),
            if stable {
                crate::data::dependency::SnapshotChangeKind::StableShapeVersionOnly
            } else {
                crate::data::dependency::SnapshotChangeKind::StructuralReplace
            }
        );
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let (mut twin, node) = fixture(stable, "scope-λ");
            let mut work = Work::new(cost + 7);
            work.reserve_visits(cost + 7 - available).unwrap();
            let result = resolve_effect_dependency_inputs(
                &mut twin,
                node,
                None,
                &mut EvaluationWork::Conditional(&mut work),
            );
            if available == cost {
                let actual = result.unwrap();
                assert_eq!(
                    actual.dependency_snapshot_update,
                    expected.dependency_snapshot_update
                );
                assert_eq!(actual.snapshot_delta, expected.snapshot_delta);
                assert_eq!(work.visits(), cost + 7);
            } else {
                assert!(matches!(
                    result,
                    Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
                ));
            }
            assert_eq!(twin.get_dep_snapshot(node).unwrap(), &before);
        }
    }
}

#[test]
fn dependency_capture_charges_scope_bytes_even_when_dependency_count_is_unchanged() {
    for stable in [false, true] {
        let (mut short, node) = fixture(stable, "x");
        let mut measured = Work::new(1_000_000);
        resolve_effect_dependency_inputs(
            &mut short,
            node,
            None,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
        let (mut long, node) = fixture(stable, &"scope-λ".repeat(2_000));
        let before = long.get_dep_snapshot(node).unwrap().clone();
        let result = resolve_effect_dependency_inputs(
            &mut long,
            node,
            None,
            &mut EvaluationWork::Conditional(&mut Work::new(measured.visits())),
        );
        assert!(matches!(
            result,
            Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
        ));
        assert_eq!(long.get_dep_snapshot(node).unwrap(), &before);
    }
}
