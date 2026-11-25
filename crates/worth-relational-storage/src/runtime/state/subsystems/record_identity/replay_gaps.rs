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
        use crate::history::data::canonical_record_allocation::RecordAllocationOrigin;
        use crate::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
        use crate::transactions::data::outcomes::conflict_class::record_allocation_denial::RecordAllocationDenial;

        let authority = RecordIdentitySubsystem::default();
        let class = RecordAllocationClass::Entity;
        let partition = PartitionId::main();
        let origin = RecordAllocationOrigin::AppendFrontier;

        // First published commit owns interleaved slots [0, 2] (gap at 1) and [4] (gap at 3).
        let r0 = authority
            .reserve_exact(0, class, partition, 0, 1, origin)
            .unwrap();
        let r2 = authority
            .reserve_exact(1, class, partition, 2, 1, origin)
            .unwrap();
        let r4 = authority
            .reserve_exact(2, class, partition, 4, 1, origin)
            .unwrap();
        assert_eq!(authority.frontier_snapshot(), vec![(class, partition, 5)]);

        // Second published commit consumes skipped interior slot 1 cleanly.
        let r1 = authority
            .reserve_exact(0, class, partition, 1, 1, origin)
            .unwrap();
        for mut res in [r0, r2, r4, r1] {
            res.consume();
        }

        // End of replay burns unpublished gap slot 3; replaying it later is refused.
        authority.release_unowned_reservations();
        let err = authority
            .reserve_exact(0, class, partition, 3, 1, origin)
            .unwrap_err();
        assert!(matches!(
            err,
            RecordAllocationDenial::ReplayAppendFrontierMismatch {
                expected_slot: 5,
                observed_slot: 3,
                ..
            }
        ));
    }
}
