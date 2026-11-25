//! One counted cell lifetime, movable independently of its immutable region.
use super::RelationalRegionResidency;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(super) struct RegionLeaseCustody {
    cell: Arc<RelationalRegionResidency>,
    active: Mutex<bool>,
}

impl RegionLeaseCustody {
    pub(super) fn accepted(cell: Arc<RelationalRegionResidency>) -> Self {
        Self {
            cell,
            active: Mutex::new(true),
        }
    }

    pub(super) fn retire_native(&self) {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *active {
            *active = false;
            let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
            debug_assert!(
                previous > 0,
                "this original lease retires its one issued count"
            );
        }
    }

    pub(super) fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }
}

impl Clone for RegionLeaseCustody {
    /// A clone shares its source's custody state. Serializing with
    /// `retire_native` guarantees the source's own count stays nonzero until
    /// this clone increments `cell.leases`, so a clone never moves `leases`
    /// from zero to one outside the cell gate. Cloning a retired custody yields
    /// another retired custody: retirement can never re-mint an
    /// eviction-blocking count.
    fn clone(&self) -> Self {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !*active {
            return Self {
                cell: self.cell.clone(),
                active: Mutex::new(false),
            };
        }
        self.cell
            .leases
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect(/* Proof: Every live lease owns a strong Arc to this cell; lease count is bounded by Arc strong count, whose maximum is below usize::MAX. */ "a resident lease count fits the address space");
        Self::accepted(self.cell.clone())
    }
}

impl Drop for RegionLeaseCustody {
    fn drop(&mut self) {
        let active = self
            .active
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !*active {
            return;
        }
        *active = false;
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(
            /* Proof: RegionLeaseCustody::accepted is private to lease admission after its count increment; Clone increments separately before constructing the copy. Each custody owns a strong Arc and Drop decrements its one issued count once. */
            previous > 0
        );
    }
}
