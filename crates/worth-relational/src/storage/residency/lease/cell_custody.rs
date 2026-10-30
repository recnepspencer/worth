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
        {
       let update_counter = &self.cell
            .leases;
       let mut count = update_counter.load(Ordering::Acquire);
       loop {
           let Some(updated_value) = count.checked_add(1) else { break Err(count) };
           match update_counter.compare_exchange_weak(count, updated_value, Ordering::AcqRel, Ordering::Acquire) {
               Ok(previous) => break Ok(previous),
               Err(reloaded_value) => count = reloaded_value,
           }
       }
   }
            .expect(/* Proof: Every live lease owns a strong Arc to this cell; lease count is bounded by Arc strong count, whose maximum is below usize::MAX. */ "a resident lease count fits the address space");
        Self::accepted(self.cell.clone())
    }
}

impl Drop for RegionLeaseCustody {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(
            /* Proof: RegionLeaseCustody::accepted is private to lease admission after its count increment; Clone increments separately before constructing the copy. Each custody owns a strong Arc and Drop decrements its one issued count once. */
            previous > 0
        );
    }
}
