//! Append-frontier slots a tail replay skipped.
//!
//! Live preparation reserves append slots in preparation order, but
//! independent branch candidates publish in FIFO reservation order, which may
//! differ. When replay meets a higher slot first, the skipped slots are kept
//! here as ranges; a later publication of a skipped slot consumes it instead
//! of being mistaken for frontier corruption. Slots never published stay
//! burned, exactly as an abandoned live reservation's gap does.
use std::collections::BTreeMap;

use super::RecordFrontierKey;

#[derive(Debug, Clone, Default)]
pub(super) struct ReplayGaps {
    /// Per frontier: disjoint `[start, end)` ranges keyed by `start`.
    ranges: BTreeMap<RecordFrontierKey, BTreeMap<usize, usize>>,
}

impl ReplayGaps {
    pub(super) fn skip(&mut self, key: RecordFrontierKey, start: usize, end: usize) {
        if start < end {
            self.ranges.entry(key).or_default().insert(start, end);
        }
    }

    /// Consume `slot` if replay skipped it.
    pub(super) fn take(&mut self, key: RecordFrontierKey, slot: usize) -> bool {
        let Some(ranges) = self.ranges.get_mut(&key) else {
            return false;
        };
        let Some((&start, &end)) = ranges.range(..=slot).next_back() else {
            return false;
        };
        if slot >= end {
            return false;
        }
        ranges.remove(&start);
        if start < slot {
            ranges.insert(start, slot);
        }
        if slot + 1 < end {
            ranges.insert(slot + 1, end);
        }
        if ranges.is_empty() {
            self.ranges.remove(&key);
        }
        true
    }

    pub(super) fn clear(&mut self) {
        self.ranges.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::data::canonical_record_allocation::RecordAllocationClass;
    use crate::identity::data::PartitionId;

    #[test]
    fn skipped_slots_are_consumed_once_and_only_inside_a_gap() {
        let key = (RecordAllocationClass::Entity, PartitionId::main());
        let mut gaps = ReplayGaps::default();
        gaps.skip(key, 1, 4);
        assert!(!gaps.take(key, 0));
        assert!(gaps.take(key, 2));
        assert!(!gaps.take(key, 2));
        assert!(gaps.take(key, 1));
        assert!(gaps.take(key, 3));
        assert!(!gaps.take(key, 4));
        assert!(gaps.ranges.is_empty());
    }

    #[test]
    fn interleaved_multi_record_replay_consumes_interior_gaps_and_burns_unpublished_slots() {
        use crate::history::data::canonical_record_allocation::{
            CanonicalRecordAllocation, RecordAllocationOrigin,
        };
        use crate::identity::data::EntityId;
        use crate::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
        use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;
        use crate::transactions::data::primitives::RecordRef;

        let authority = RecordIdentitySubsystem::default();
        let class = RecordAllocationClass::Entity;
        let partition = PartitionId::main();
        let origin = RecordAllocationOrigin::AppendFrontier;
        let live = |ordinal, slot| {
            let record = RecordRef::Entity(EntityId::new(partition, slot, 1));
            CanonicalRecordAllocation::with_origin(ordinal, record, origin)
        };
        let replay = |slots: &[u64]| {
            let evidence = slots.iter().enumerate();
            let evidence = evidence.map(|(ordinal, &slot)| live(ordinal as u64, slot));
            authority
                .stage_replay_allocations(evidence.collect(), Vec::new())
                .unwrap();
            let reserved = slots
                .iter()
                .enumerate()
                .map(|(ordinal, &slot)| {
                    authority
                        .reserve_exact(ordinal as u64, class, partition, slot as usize, 1, origin)
                        .unwrap()
                })
                .collect::<Vec<_>>();
            authority.clear_staged_replay_allocations();
            reserved
        };

        // First published commit owns interleaved slots [0, 2] (gap at 1) and [4] (gap at 3).
        let first = replay(&[0, 2, 4]);
        assert_eq!(authority.frontier_snapshot(), vec![(class, partition, 5)]);

        // Second published commit consumes skipped interior slot 1 cleanly.
        let second = replay(&[1]);
        for mut res in first.into_iter().chain(second) {
            res.consume();
        }

        // End of replay burns unpublished gap slot 3; replaying it later is refused.
        authority.release_unowned_reservations();
        let err = authority
            .stage_replay_allocations(vec![live(0, 3)], Vec::new())
            .unwrap_err();
        assert!(matches!(
            err,
            RecordAllocationDenial::ReplayEvidenceBlockBroken {
                expected_slot: 5,
                observed_slot: 3,
                ..
            }
        ));
    }
}
