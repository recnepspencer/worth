use super::*;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn replacement_preparation_shares_one_allowance_and_preserves_source_on_denial() {
    let source = DependencySnapshot::from_ordered_unique((0..8).map(|n| DependencySnapshotEntry {
        source: NodeId::new(n, 0),
        aspect: Aspect::new(0),
        cached_version: u64::from(n),
        scope: Some(PartitionSubscription::partition_and_detail(
            "partition".repeat(32),
            "detail".repeat(32),
        )),
    }));
    let mut measured = Work::new(100_000_000);
    let expected = ReplacementSnapshotUpdate::from_snapshot_with_work(
        source.clone(),
        &mut EvaluationWork::Conditional(&mut measured),
    )
    .unwrap();
    let cost = measured.visits();
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 19);
        work.reserve_visits(cost + 19 - available).unwrap();
        let result = ReplacementSnapshotUpdate::from_snapshot_with_work(
            source.clone(),
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            let result = result.unwrap();
            assert_eq!(result, expected);
            assert!(result.snapshot().snapshot().shares_storage_with(&source));
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 19
                })
            );
        }
        assert_eq!(source.entries().len(), 8);
    }
}
