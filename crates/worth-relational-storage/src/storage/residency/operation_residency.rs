//! Operation-scoped residency custody (Store M4 D1).
//!
//! Roots and editions retain only durable member custody. Every lease an
//! operation admits lives here, so the read views it lends are bounded by the
//! operation that owns this value: dropping it releases each counted borrow and
//! its strong `Arc<ResidentRegion>`, and a bound cell becomes evictable again.
//!
//! Slots are append-only and never move, so a lent view stays valid for as long
//! as the scope is borrowed. Slots are keyed by cell identity, which the slot
//! itself pins, so one scope can serve several roots that share cells.
use super::{Full, Head, RelationalRegionResidency, RelationalResidencyLease};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

const FIRST_SEGMENT: usize = 8;
const SEGMENTS: usize = 48;

#[derive(Debug, Default)]
pub(super) struct ScopedRegion {
    cell: OnceLock<Arc<RelationalRegionResidency>>,
    pub(super) full: OnceLock<RelationalResidencyLease<Full>>,
    pub(super) head: OnceLock<RelationalResidencyLease<Head>>,
}

#[derive(Debug)]
pub struct OperationResidency {
    positions: Mutex<BTreeMap<usize, usize>>,
    segments: [OnceLock<Box<[ScopedRegion]>>; SEGMENTS],
}

impl Default for OperationResidency {
    fn default() -> Self {
        Self::new()
    }
}

fn locate(position: usize) -> (usize, usize) {
    let segment = (position / FIRST_SEGMENT + 1).ilog2() as usize;
    let start = FIRST_SEGMENT * ((1 << segment) - 1);
    (segment, position - start)
}

impl OperationResidency {
    pub fn new() -> Self {
        Self {
            positions: Mutex::new(BTreeMap::new()),
            segments: std::array::from_fn(|_| OnceLock::new()),
        }
    }

    /// Distinct cells this operation has touched.
    pub fn len(&self) -> usize {
        self.positions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn at(&self, position: usize) -> &ScopedRegion {
        let (segment, offset) = locate(position);
        let slots = self.segments[segment].get_or_init(|| {
            (0..FIRST_SEGMENT << segment)
                .map(|_| ScopedRegion::default())
                .collect()
        });
        &slots[offset]
    }

    fn cell_key(cell: &Arc<RelationalRegionResidency>) -> usize {
        #[expect(
            clippy::disallowed_methods,
            reason = "The slot pins the allocation used as its lookup key; order is never observed"
        )]
        let key = Arc::as_ptr(cell) as usize;
        key
    }

    /// The slot already holding custody for `cell`, if this operation has one.
    pub(super) fn existing(&self, cell: &Arc<RelationalRegionResidency>) -> Option<&ScopedRegion> {
        let key = Self::cell_key(cell);
        let position = *self
            .positions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&key)?;
        Some(self.at(position))
    }

    /// The slot for `cell`, allocated on first use. The slot pins the cell, so
    /// its address key cannot be reused while this scope lives.
    pub(super) fn slot(&self, cell: &Arc<RelationalRegionResidency>) -> &ScopedRegion {
        let key = Self::cell_key(cell);
        let mut positions = self
            .positions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let next = positions.len();
        let position = *positions.entry(key).or_insert(next);
        let slot = self.at(position);
        let _ = slot.cell.set(cell.clone());
        drop(positions);
        slot
    }
}

#[cfg(test)]
mod tests {
    use super::locate;

    #[test]
    fn positions_fill_doubling_segments_without_gaps() {
        assert_eq!(locate(0), (0, 0));
        assert_eq!(locate(7), (0, 7));
        assert_eq!(locate(8), (1, 0));
        assert_eq!(locate(23), (1, 15));
        assert_eq!(locate(24), (2, 0));
    }
}
