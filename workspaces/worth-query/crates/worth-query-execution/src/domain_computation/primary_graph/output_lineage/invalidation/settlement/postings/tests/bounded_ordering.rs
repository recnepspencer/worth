//! Adversarial order, exact collision grouping and fallible admission.

use std::{collections::BTreeMap, sync::Arc};

use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    mvcc::{
        CompanionPreflightBudget, CompanionPreflightStop, RelationalCancellationSource,
        RelationalInterruptionBoundary, RelationalOperationControl,
        RelationalOperationInterruption,
    },
};

use super::super::{
    group_projected, FactPostingKey, IndexAdmission, InvalidationEditAdmission,
    PreparedPostingOrdinals, ProjectedKey, WorthQueryApplicationObservedFact as Fact,
};

fn key(slot: usize) -> Arc<FactPostingKey> {
    Arc::new(FactPostingKey::EntityLifecycle(EntityId::new(
        PartitionId::main(),
        slot as u64 + 1,
        1,
    )))
}

fn admission(work: u64, bytes: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(work),
        maximum_preparation_bytes: Some(bytes),
    })
}

#[test]
fn reverse_mixed_and_colliding_orders_fit_n_log_n_work_and_keep_every_ordinal() {
    for count in [128usize, 1024, 4096] {
        for order in 0..3 {
            let mut expected = BTreeMap::<_, Vec<_>>::new();
            let projected = (0..count)
                .map(|ordinal| {
                    let selected = match order {
                        0 => count - ordinal - 1,
                        1 => ordinal * 73 % count,
                        _ => (ordinal * 73 % count) % 64,
                    };
                    let key = key(selected);
                    expected.entry(Arc::clone(&key)).or_default().push(ordinal);
                    ProjectedKey {
                        hash: if order == 2 { 7 } else { selected as u64 },
                        key,
                        ordinal,
                    }
                })
                .collect();
            // Generous growth-class bound: reverse insertion or a linear scan
            // of distinct colliding keys cannot fit this as population grows.
            let ceiling = 64 * count as u64 * u64::from(count.ilog2() + 1);
            let mut meter = admission(ceiling, u64::MAX);
            let groups = group_projected(projected, count, &mut meter).unwrap();
            let actual: BTreeMap<_, _> = groups
                .into_iter()
                .map(|group| {
                    (
                        group.key,
                        group.ordinals.iter().copied().collect::<Vec<_>>(),
                    )
                })
                .collect();
            assert_eq!(actual, expected, "count={count}, order={order}");
            assert!(meter.charged_work() <= ceiling);
        }
    }
}

#[test]
fn a_large_single_hash_bucket_has_bounded_exact_key_ordering() {
    let count = 4096usize;
    let projected = (0..count)
        .map(|ordinal| ProjectedKey {
            hash: 7,
            key: key(count - ordinal - 1),
            ordinal,
        })
        .collect();
    let mut meter = admission(64 * count as u64 * 13, u64::MAX);
    let groups = group_projected(projected, count, &mut meter).unwrap();
    assert_eq!(groups.len(), count);
    for (slot, group) in groups.iter().enumerate() {
        assert_eq!(group.key, key(slot));
        assert_eq!(
            group.ordinals.iter().copied().collect::<Vec<_>>(),
            [count - slot - 1]
        );
    }
}

#[test]
fn failed_work_and_storage_admission_return_without_retaining_projected_keys() {
    let shared = key(0);
    for ceiling in [0, 12, 256] {
        let projected = (0..128)
            .map(|ordinal| ProjectedKey {
                hash: 127 - ordinal as u64,
                key: Arc::clone(&shared),
                ordinal,
            })
            .collect();
        let mut meter = admission(ceiling, u64::MAX);
        assert!(matches!(
            group_projected(projected, 128, &mut meter),
            Err(CompanionPreflightStop::WorkExhausted { .. })
        ));
        assert!(meter.charged_work() <= ceiling);
        assert_eq!(Arc::strong_count(&shared), 1);
        assert_eq!(meter.charged_index_bytes(), 0);
    }
    let facts = [Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), 1, 1),
    }];
    let mut meter = admission(10_000, 0);
    assert!(matches!(
        PreparedPostingOrdinals::prepare(&facts, None, &mut None, &mut meter),
        Err(CompanionPreflightStop::PreparationMemoryExhausted { .. })
    ));
    assert_eq!(meter.charged_bytes(), 0);
    assert_eq!(meter.charged_index_bytes(), 0);
}

/// A focused adapter observes real native cancellation during the fallible
/// algorithm. It tests propagation, not a new production cancellation owner.
struct CancellableAdmission {
    meter: InvalidationEditAdmission,
    source: RelationalCancellationSource,
    control: RelationalOperationControl,
    accepted_calls: usize,
}

impl IndexAdmission for CancellableAdmission {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        if self.accepted_calls == 32 {
            self.source.cancel();
        }
        if let Some(event) = self
            .control
            .observe(RelationalInterruptionBoundary::PublicationPreflight)
        {
            return Err(CompanionPreflightStop::Interrupted(event));
        }
        self.meter.work(visits)?;
        self.accepted_calls += 1;
        Ok(())
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        self.meter.bytes(bytes)
    }

    fn navigation(&mut self, units: u64) -> Result<(), CompanionPreflightStop> {
        self.meter.navigation(units)
    }
}

#[test]
fn cancellation_during_ordering_returns_the_native_stop_and_drops_scratch() {
    let source = RelationalCancellationSource::new();
    let mut meter = CancellableAdmission {
        meter: admission(1_000_000, u64::MAX),
        control: source.token().into(),
        source,
        accepted_calls: 0,
    };
    let shared = key(0);
    let projected = (0..128)
        .map(|ordinal| ProjectedKey {
            hash: 127 - ordinal as u64,
            key: Arc::clone(&shared),
            ordinal,
        })
        .collect();
    let Err(CompanionPreflightStop::Interrupted(event)) =
        group_projected(projected, 128, &mut meter)
    else {
        panic!("ordering must return the original native cancellation");
    };
    assert_eq!(
        event.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        event.boundary(),
        RelationalInterruptionBoundary::PublicationPreflight
    );
    assert!(meter.meter.charged_work() > 0);
    assert!(meter.meter.charged_work() < 1_000_000);
    assert_eq!(meter.meter.charged_index_bytes(), 0);
    assert_eq!(Arc::strong_count(&shared), 1);
}
