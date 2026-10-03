//! Retained head observations move with their exact native ownership.

use super::{
    backing::{vector_bytes, HeadWalkBacking},
    Denial, SelectedArtifactSlice,
};
use crate::physical_runtime::PhysicalRecoveryReadAllocation;

pub(in crate::physical_runtime) struct FundedHeadSlices {
    slices: Vec<SelectedArtifactSlice>,
    backing: HeadWalkBacking,
}

impl FundedHeadSlices {
    pub(super) fn new(slices: Vec<SelectedArtifactSlice>, backing: HeadWalkBacking) -> Self {
        Self { slices, backing }
    }

    pub(in crate::physical_runtime) fn slices(&self) -> &[SelectedArtifactSlice] {
        &self.slices
    }
    pub(in crate::physical_runtime) fn owned_heap_bytes(&self) -> Option<u64> {
        vector_bytes(&self.slices).ok()
    }
    #[cfg(test)]
    pub(in crate::physical_runtime) fn charged_bytes(&self) -> u64 {
        self.backing.charged_bytes()
    }
    pub(in crate::physical_runtime) fn matching_owner(
        &self,
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> bool {
        self.backing.matching_owner(window)
    }
}

impl super::ObservedReleaseHeads {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_funded_slices(
        self,
    ) -> Result<FundedHeadSlices, Denial> {
        let mut backing = self.backing;
        let entries = self.entries;
        let slices = self.slices;
        drop(entries);
        backing.settle(vector_bytes(&slices)?)?;
        Ok(FundedHeadSlices::new(slices, backing))
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_funded_slices_with_resident(
        self,
        resident: &mut super::StoreRejoinResidentLedger,
    ) -> Result<FundedHeadSlices, Denial> {
        let discarded = vector_bytes(&self.entries)?;
        let funded = self.into_funded_slices()?;
        resident.release(discarded);
        Ok(funded)
    }
}
