//! Required native WAL backing travels with the complete source selection.

use super::resident_allocation::{vector_bytes, WalSelectionAllocation};
use std::ops::Deref;
use worth_store_recovery_physics::PhysicalSourceSelection;

#[derive(Debug)]
pub(crate) struct ResidentSourceSelection {
    selection: Vec<PhysicalSourceSelection>,
    allocation: WalSelectionAllocation,
}

impl ResidentSourceSelection {
    pub(super) fn finish(
        mut backing: Vec<PhysicalSourceSelection>,
        selection: PhysicalSourceSelection,
        mut allocation: WalSelectionAllocation,
        previously_retained: u64,
    ) -> Result<Self, worth_store::physical_runtime::RecoveryWalAllocationDenial> {
        assert!(backing.is_empty() && backing.capacity() == 1);
        let retained = previously_retained
            .checked_add(vector_bytes::<PhysicalSourceSelection>(backing.capacity())?)
            .ok_or(worth_store::physical_runtime::RecoveryWalAllocationDenial::SizeOverflow)?;
        backing.push(selection);
        // The actual payload is now in the admitted slot. Only construction
        // storage is released; existing tail backing remains charged.
        allocation.settle_after_disposal(retained)?;
        Ok(Self {
            selection: backing,
            allocation,
        })
    }

    pub(crate) const fn facts(&self) -> &PhysicalSourceSelection {
        &self.selection.as_slice()[0]
    }
    pub(crate) fn charged_bytes(&self) -> u64 {
        self.allocation.charged_bytes()
    }

    pub(crate) fn construction_peak_bytes(&self) -> u64 {
        self.allocation.peak_bytes()
    }

    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        vector_bytes::<PhysicalSourceSelection>(self.selection.capacity())
            .ok()?
            .checked_add(self.facts().owned_heap_bytes()?)
    }
}

impl Deref for ResidentSourceSelection {
    type Target = PhysicalSourceSelection;
    fn deref(&self) -> &Self::Target {
        self.facts()
    }
}
