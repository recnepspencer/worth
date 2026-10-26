//! One counted cell lifetime, movable independently of its immutable region.
use super::RelationalRegionResidency;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct RegionLeaseCustody {
    cell: Arc<RelationalRegionResidency>,
}

impl RegionLeaseCustody {
    pub(super) fn accepted(cell: Arc<RelationalRegionResidency>) -> Self {
        Self { cell }
    }

    pub(super) fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }
}

impl Clone for RegionLeaseCustody {
    fn clone(&self) -> Self {
        self.cell
            .leases
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect("a resident lease count fits the address space");
        Self::accepted(self.cell.clone())
    }
}

impl Drop for RegionLeaseCustody {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0);
    }
}
