use super::allocation_inventory::{
    RelationalAuthoritativeAllocationObservation, RelationalStorageRegionLocator,
};
use super::observation::RelationalBranchSharingObservation;
use super::root_commitment::{
    RelationalCorrectnessIndexPosture, RelationalVisibilityCommitmentObservation,
};

/// Live authoritative evidence: the identities and commitments behind the byte
/// totals, produced by the same owner walk at the same observation time.
///
/// This lane exists so that the byte totals are checkable rather than
/// asserted. Nothing here is a recorded counter, and nothing here opens
/// authority over the objects it names.
impl RelationalBranchSharingObservation {
    /// Identities of the distinct storage regions holding authoritative
    /// partition payloads.
    ///
    /// Truth source: the owner walk, restricted to allocations whose kind is
    /// [`RelationalAuthoritativeAllocationKind::RootRegionObject`](super::allocation_inventory::RelationalAuthoritativeAllocationKind::RootRegionObject).
    /// Equal regions are collapsed. Distinct regions may still share payload
    /// allocations; payload byte deduplication uses allocation identities.
    pub fn region_locators(&self) -> &[RelationalStorageRegionLocator] {
        &self.region_locators
    }

    /// Every distinct authoritative allocation reached from the selection,
    /// with its own byte count.
    ///
    /// Truth source: the complete owner walk, deduplicated by locator and
    /// ordered by locator. Summing
    /// [`RelationalAuthoritativeAllocationObservation::authoritative_bytes`]
    /// over this slice reproduces
    /// [`Self::unique_physical_authoritative_bytes`] exactly, which is what
    /// makes that total independently checkable.
    pub fn authoritative_allocations(&self) -> &[RelationalAuthoritativeAllocationObservation] {
        &self.authoritative_allocations
    }

    /// One visibility commitment per distinct selected root.
    ///
    /// Truth source: each distinct root's resolved visible axes, digested at
    /// observation time. Branches that share a root share one commitment, so
    /// this slice has [`Self::unique_root_count`] entries rather than
    /// [`Self::branch_count`] entries.
    pub fn visibility_commitments(&self) -> &[RelationalVisibilityCommitmentObservation] {
        &self.visibility_commitments
    }

    /// How correctness answers are served for the selected roots.
    ///
    /// Truth source: the resolved correctness-index axis of the selected
    /// roots, read live. It is a posture, not a byte count or a counter.
    pub const fn correctness_index_posture(&self) -> RelationalCorrectnessIndexPosture {
        self.correctness_index_posture
    }

    /// Number of storage regions the inspection walked to build this
    /// observation.
    ///
    /// Truth source: the inspection walk itself, summed once per distinct
    /// selected root. It measures the cost of observing, not the storage that
    /// was observed, and it is not comparable with the recorded publication
    /// region counts, which are written by publication work rather than by
    /// inspection.
    pub const fn inspection_reconstructed_region_count(&self) -> u64 {
        self.inspection_reconstructed_region_count
    }
}
