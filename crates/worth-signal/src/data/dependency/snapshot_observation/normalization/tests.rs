use super::*;
use crate::data::aspect::Aspect;
use crate::data::dependency::{DependencySnapshotShapeStore, DependencySnapshotStore};
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use std::collections::BTreeMap;

fn entry(n: usize) -> DependencySnapshotEntry {
    DependencySnapshotEntry {
        source: NodeId::new((n % 5) as u32, (n % 3) as u32),
        aspect: Aspect::new((n % 3) as u8),
        cached_version: (n % 11) as u64,
        scope: match n % 3 {
            0 => None,
            1 => Some(PartitionSubscription::whole_partition("scope-λ".repeat(20))),
            _ => Some(PartitionSubscription::partition_and_detail(
                "scope",
                "detail-λ".repeat(20),
            )),
        },
    }
}

#[test]
fn normalization_matches_independent_map_and_preserves_shared_input() {
    for count in 0..128 {
        let entries: Vec<_> = (0..count).rev().map(entry).collect();
        let mut expected = BTreeMap::new();
        for value in &entries {
            expected
                .entry(value.sort_key())
                .and_modify(|v: &mut u64| *v = (*v).max(value.cached_version))
                .or_insert(value.cached_version);
        }
        // Explicit unordered fixture: do not assert canonicality at construction.
        let source = DependencySnapshot {
            entries: Arc::new(entries.clone()),
        };
        let normalized = source.clone().canonicalize_unordered();
        assert_eq!(source.entries(), entries);
        assert_eq!(normalized.entries().len(), expected.len());
        for (actual, (key, version)) in normalized.entries().iter().zip(expected) {
            assert_eq!(actual.sort_key(), key);
            assert_eq!(actual.cached_version, version);
        }
        let repeated = normalized.clone().canonicalize_unordered();
        assert!(repeated.shares_storage_with(&normalized));
    }
}

#[test]
fn snapshot_store_reuses_canonical_backing_and_retains_unique_shape_handles() {
    let source = DependencySnapshot {
        entries: Arc::new((0..32).map(entry).collect()),
    }
    .canonicalize_unordered();
    let mut snapshots = DependencySnapshotStore::default();
    let mut shapes = DependencySnapshotShapeStore::default();
    let (id, shape) = snapshots.insert_with_shape_handle(source.clone(), &mut shapes);
    assert!(snapshots.get(id).shares_storage_with(&source));
    assert_eq!(snapshots.get(id), &source);
    assert_eq!(
        snapshots.insert_with_shape_handle(source.clone(), &mut shapes),
        (id, shape)
    );
    assert_eq!(snapshots.snapshot_count(), 1);
    assert_eq!(snapshots.require_retained_indexes(&shapes), Ok(()));
}

#[test]
fn borrowed_key_comparison_matches_owned_key_order_independently_of_version() {
    for left in 0..32 {
        for right in 0..32 {
            let mut a = entry(left);
            let b = entry(right);
            a.cached_version = u64::MAX;
            assert_eq!(a.compare_key(&b), a.sort_key().cmp(&b.sort_key()));
        }
    }
}

#[test]
fn normalization_uses_shared_work_before_copying_or_sorting() {
    use crate::data::retained_storage::RetainedStoragePreparation as Work;
    use crate::logic::evaluation::EvaluationWork;
    for canonical in [false, true] {
        let source = DependencySnapshot {
            entries: Arc::new((0..64).rev().map(entry).collect()),
        };
        let source = if canonical {
            source.canonicalize_unordered()
        } else {
            source
        };
        let before = source.entries().to_vec();
        let mut measured = Work::new(100_000_000);
        let expected = source
            .clone()
            .canonicalize_with_work(&mut EvaluationWork::Conditional(&mut measured))
            .unwrap();
        let cost = measured.visits();
        for available in [cost - 1, cost] {
            let mut work = Work::new(cost + 11);
            work.reserve_visits(cost + 11 - available).unwrap();
            let result = source
                .clone()
                .canonicalize_with_work(&mut EvaluationWork::Conditional(&mut work));
            if available == cost {
                let result = result.unwrap();
                assert_eq!(result, expected);
                if canonical {
                    assert!(result.shares_storage_with(&source));
                }
            } else {
                assert_eq!(
                    result,
                    Err(
                        crate::data::error::SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: cost + 11
                        }
                    )
                );
            }
            assert_eq!(source.entries(), before);
        }
    }
}
