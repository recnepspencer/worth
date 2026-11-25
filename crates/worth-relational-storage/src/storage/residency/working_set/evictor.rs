//! Clock-order victim selection that never blocks on a cell `state` mutex.
//!
//! The ring lock is a leaf: it is held only to snapshot or append weak cell
//! handles, never while trying a victim or dropping its contents. Victims are
//! taken with `try_lock` alone, so an evictor holding its own demand target's
//! gate can never join an AB/BA cycle with another evictor.
use super::{ResidencyPoolKind, WorkingSetAccountant};
use crate::storage::residency::cell::RegionCellWeak;
use crate::storage::residency::RelationalRegionResidency;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(in crate::internal_port::storage::residency) struct ClockRing {
    cells: Vec<RegionCellWeak>,
    hand: usize,
}

/// Victims one sweep evicted or skipped as busy.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SweepOutcome {
    pub(super) evicted: usize,
    pub(super) busy: usize,
}

/// Outcome of one non-blocking eviction attempt on a candidate cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionAttempt {
    Evicted,
    /// Another thread holds the cell gate or one of its raw caches.
    Busy,
    /// A live counted borrow, or a recent reference given a second chance.
    Borrowed,
    /// Cold, damaged, unbacked, foreign to this home, or outside this pool.
    Ineligible,
}

impl WorkingSetAccountant {
    /// Register a bound cell. The ring retains only a weak handle.
    pub(in crate::internal_port::storage::residency) fn enroll(&self, cell: RegionCellWeak) {
        let mut ring = self.ring.lock().unwrap_or_else(|error| error.into_inner());
        ring.cells.retain(|entry| entry.upgrade().is_some());
        ring.cells.push(cell);
    }

    pub(super) fn live_cells(&self) -> Vec<Arc<RelationalRegionResidency>> {
        let ring = self.ring.lock().unwrap_or_else(|error| error.into_inner());
        ring.cells
            .iter()
            .filter_map(RegionCellWeak::upgrade)
            .collect()
    }

    /// Evict unborrowed victims in clock order until `bytes` fit in `pool` or
    /// two full sweeps (the second-chance round) find nothing more to free.
    pub(super) fn evict_for(
        self: &Arc<Self>,
        pool: ResidencyPoolKind,
        bytes: u64,
        requester: *const RelationalRegionResidency,
    ) -> SweepOutcome {
        let mut sweep = SweepOutcome::default();
        let (candidates, start) = {
            let mut ring = self.ring.lock().unwrap_or_else(|error| error.into_inner());
            ring.cells.retain(|entry| entry.upgrade().is_some());
            let start = if ring.cells.is_empty() {
                0
            } else {
                ring.hand % ring.cells.len()
            };
            (ring.cells.clone(), start)
        };
        let ledger = self.ledger(super::LedgerKind::Pool(pool));
        let total = candidates.len();
        let mut visited = 0;
        while visited < total.saturating_mul(2) && ledger.available() < bytes {
            let entry = &candidates[(start + visited) % total];
            visited += 1;
            if entry.as_ptr() == requester {
                continue;
            }
            let Some(cell) = entry.upgrade() else {
                continue;
            };
            match cell.try_evict(self, pool) {
                EvictionAttempt::Evicted => {
                    sweep.evicted += 1;
                    self.evictions.fetch_add(1, Ordering::AcqRel);
                }
                EvictionAttempt::Busy => {
                    sweep.busy += 1;
                    self.busy_skips.fetch_add(1, Ordering::AcqRel);
                }
                EvictionAttempt::Borrowed | EvictionAttempt::Ineligible => {}
            }
            // `cell` drops here, outside every lock this evictor took.
        }
        let mut ring = self.ring.lock().unwrap_or_else(|error| error.into_inner());
        ring.hand = ring.hand.wrapping_add(visited);
        sweep
    }
}
