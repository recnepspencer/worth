use super::*;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

fn shape(label: &str) -> DependencySnapshotShape {
    DependencySnapshotShape::from_ordered_unique([DependencySnapshotEntry {
        source: NodeId::new(1, 0),
        aspect: Aspect::new(0),
        cached_version: 3,
        scope: Some(PartitionSubscription::whole_partition(label)),
    }
    .sort_key()])
}

#[test]
fn shape_interning_admits_existing_and_new_handles_before_mutation() {
    for existing in [false, true] {
        let query = shape("query");
        let mut source = DependencySnapshotShapeStore::default();
        let original = source.intern(shape("original"));
        if existing {
            source.intern(query.clone());
        }
        let mut measured_store = source.fork_persistent();
        let mut measured = Work::new(10_000_000);
        let expected = measured_store
            .intern_with_work(
                query.clone(),
                &mut EvaluationWork::Conditional(&mut measured),
            )
            .unwrap();
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let mut candidate = source.fork_persistent();
            let before = candidate.clone();
            let mut work = Work::new(cost + 7);
            work.reserve_visits(cost + 7 - available).unwrap();
            let result = candidate
                .intern_with_work(query.clone(), &mut EvaluationWork::Conditional(&mut work));
            if available == cost {
                assert_eq!(result.unwrap(), expected);
                assert_eq!(candidate.intern(query.clone()), expected);
            } else {
                assert_eq!(
                    result,
                    Err(SignalError::ConditionalEvaluationWorkExhausted {
                        maximum_visits: cost + 7
                    })
                );
                assert_eq!(candidate, before);
            }
            assert_eq!(source.intern(shape("original")), original);
            assert_eq!(source.shapes.len(), if existing { 2 } else { 1 });
        }
    }
}

#[test]
fn shape_interning_charges_payload_bytes_and_denies_missing_index() {
    let mut store = DependencySnapshotShapeStore::default();
    store.intern(shape("seed"));
    let mut short = store.fork_persistent();
    let mut measured = Work::new(10_000_000);
    short
        .intern_with_work(shape("x"), &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let mut long = store.fork_persistent();
    let before = long.clone();
    let mut limited = Work::new(measured.visits());
    assert!(matches!(
        long.intern_with_work(
            shape(&"x".repeat(4096)),
            &mut EvaluationWork::Conditional(&mut limited)
        ),
        Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
    ));
    assert_eq!(long, before);
    let mut restored: DependencySnapshotShapeStore =
        serde_json::from_str(&serde_json::to_string(&store).unwrap()).unwrap();
    assert!(!restored.retained_interner_is_complete());
    assert_eq!(
        restored.intern_with_work(
            shape("seed"),
            &mut EvaluationWork::Conditional(&mut Work::new(10_000_000))
        ),
        Err(SignalError::SnapshotIndexUnavailable)
    );
    assert!(!restored.retained_interner_is_complete());
    assert_eq!(restored.intern(shape("seed")), store.intern(shape("seed")));
}
