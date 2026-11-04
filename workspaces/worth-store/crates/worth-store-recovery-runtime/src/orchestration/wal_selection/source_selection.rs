//! Required native WAL backing travels with the complete source selection.

use super::resident_allocation::{vector_bytes, WalSelectionAllocation};
use std::ops::Deref;
use worth_store_recovery_physics::PhysicalSourceSelection;

#[derive(Debug)]
pub(crate) struct ResidentSourceSelection {
    selection: Vec<PhysicalSourceSelection>,
    allocation: WalSelectionAllocation,
}

// Field order keeps every source payload alive only while its native grant lives.
struct SourceSelectionInput {
    selection: PhysicalSourceSelection,
    allocation: WalSelectionAllocation,
}

impl ResidentSourceSelection {
    pub(super) fn capture(
        selection: PhysicalSourceSelection,
        allocation: WalSelectionAllocation,
        owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    ) -> Result<Self, worth_store::physical_runtime::RecoveryWalAllocationDenial> {
        use worth_store::physical_runtime::RecoveryWalAllocationDenial;
        let mut input = SourceSelectionInput {
            selection,
            allocation,
        };
        let previously_retained = input.allocation.charged_bytes();
        let construction_bytes = u64::try_from(std::mem::size_of_val(&input))
            .ok()
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<PhysicalSourceSelection>() as u64)
            })
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        let mut backing = input.allocation.prepare_vector::<PhysicalSourceSelection>(
            owner,
            1,
            construction_bytes,
        )?;
        // Exact admitted capacity is one; this move cannot grow the allocation.
        backing.push(input.selection);
        let mut captured = Self {
            selection: backing,
            allocation: input.allocation,
        };
        let retained = previously_retained
            .checked_add(vector_bytes::<PhysicalSourceSelection>(
                captured.selection.capacity(),
            )?)
            .ok_or(RecoveryWalAllocationDenial::SizeOverflow)?;
        // Every fallible step retains an aggregate whose payload drops before
        // its grant. Only construction bytes are released after filling.
        captured.allocation.settle_after_disposal(retained)?;
        Ok(captured)
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
