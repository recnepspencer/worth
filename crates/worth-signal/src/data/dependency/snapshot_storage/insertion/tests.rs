use super::*;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

fn snapshot(version: u64, scope: &str) -> DependencySnapshot {
    DependencySnapshot::from_ordered_unique([DependencySnapshotEntry {
        source: NodeId::new(1, 0),
        aspect: Aspect::new(0),
        cached_version: version,
        scope: Some(PartitionSubscription::partition_and_detail(scope, "detail")),
    }])
}

#[test]
fn snapshot_insertion_preparation_is_read_only_until_consumed() {
    for (version, scope) in [(1, "seed"), (2, "seed"), (2, "other")] {
        let mut source = DependencySnapshotStore::default();
        let mut source_shapes = DependencySnapshotShapeStore::default();
        let original = source.insert_with_shape_handle(snapshot(1, "seed"), &mut source_shapes);
        let query = snapshot(version, scope);
        let mut measured_store = source.fork_persistent();
        let mut measured_shapes = source_shapes.fork_persistent();
        let mut measured = Work::new(100_000_000);
        let prepared = measured_store
            .prepare_insertion(
                query.clone(),
                &mut measured_shapes,
                &mut EvaluationWork::Conditional(&mut measured),
            )
            .unwrap();
        assert_eq!(measured_store, source);
        assert_eq!(measured_shapes, source_shapes);
        drop(prepared);
        assert_eq!(measured_store, source);
        assert_eq!(measured_shapes, source_shapes);
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let mut candidate = source.fork_persistent();
            let mut candidate_shapes = source_shapes.fork_persistent();
            let mut work = Work::new(cost + 17);
            work.reserve_visits(cost + 17 - available).unwrap();
            let result = candidate.prepare_insertion(
                query.clone(),
                &mut candidate_shapes,
                &mut EvaluationWork::Conditional(&mut work),
            );
            assert_eq!(candidate, source);
            assert_eq!(candidate_shapes, source_shapes);
            if available == cost {
                let (id, shape) = result
                    .unwrap()
                    .publish(&mut candidate, &mut candidate_shapes);
                assert_eq!(candidate.get(id), &query);
                assert_eq!(
                    candidate.insert_with_shape_handle(query.clone(), &mut candidate_shapes),
                    (id, shape)
                );
                assert_eq!(shape == original.1, scope == "seed");
                assert_eq!(id == original.0, version == 1);
                assert_eq!(
                    candidate.require_retained_indexes(&candidate_shapes),
                    Ok(())
                );
            } else {
                assert!(
                    matches!(result, Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }) if maximum_visits == cost + 17)
                );
            }
            assert_eq!(source.snapshot_count(), 1);
            assert_eq!(source.get(original.0), &snapshot(1, "seed"));
        }
    }
}

#[test]
fn snapshot_insertion_rejects_missing_indexes_and_charges_query_payload() {
    let mut source = DependencySnapshotStore::default();
    let mut shapes = DependencySnapshotShapeStore::default();
    source.insert_with_shape_handle(snapshot(1, "seed"), &mut shapes);
    let mut short_store = source.fork_persistent();
    let mut short_shapes = shapes.fork_persistent();
    let mut measured = Work::new(100_000_000);
    short_store
        .prepare_insertion(
            snapshot(2, "x"),
            &mut short_shapes,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    let mut long_store = source.fork_persistent();
    let mut long_shapes = shapes.fork_persistent();
    assert!(matches!(
        long_store.prepare_insertion(
            snapshot(2, &"x".repeat(4096)),
            &mut long_shapes,
            &mut EvaluationWork::Conditional(&mut Work::new(measured.visits()))
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    assert_eq!(long_store, source);
    assert_eq!(long_shapes, shapes);
    let mut restored: DependencySnapshotStore =
        serde_json::from_str(&serde_json::to_string(&source).unwrap()).unwrap();
    let before = restored.clone();
    assert!(matches!(
        restored.prepare_insertion(
            snapshot(2, "seed"),
            &mut shapes,
            &mut EvaluationWork::Conditional(&mut Work::new(100_000_000))
        ),
        Err(SignalError::SnapshotIndexUnavailable)
    ));
    assert_eq!(restored, before);
    let (id, _) = restored.insert_with_shape_handle(snapshot(2, "seed"), &mut shapes);
    assert_eq!(restored.get(id), &snapshot(2, "seed"));
    assert_eq!(restored.require_retained_indexes(&shapes), Ok(()));
}
