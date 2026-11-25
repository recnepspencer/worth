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
}
