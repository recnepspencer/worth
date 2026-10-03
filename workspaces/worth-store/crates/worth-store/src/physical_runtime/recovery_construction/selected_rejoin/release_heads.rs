//! Independent native-backed walk of the selected head tree.

use super::control_frames::SelectedArtifactSlice;
use super::resident::StoreRejoinResidentLedger;
use super::SelectedMediaRejoinDenial as Denial;
use crate::physical_runtime::{
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryReadAllocation,
};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    ReleaseCustodyHeadEntryV1,
};

mod backing;
mod controls;
mod funded_slices;
mod observation;
mod port;
mod storage;
pub(super) use controls::{
    observe_controls, observe_controls_on_routes_with_resident, ObservedReleaseHeadControls,
};
pub(in crate::physical_runtime) use funded_slices::FundedHeadSlices;
#[cfg(test)]
mod tests;

pub(super) struct ObservedReleaseHeads {
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    slices: Vec<SelectedArtifactSlice>,
    count: u64,
    digest: [u8; 32],
    backing: backing::HeadWalkBacking,
}

impl ObservedReleaseHeads {
    pub(super) fn entries(&self) -> &[ReleaseCustodyHeadEntryV1] {
        &self.entries
    }
    #[cfg(test)]
    pub(super) fn count(&self) -> u64 {
        self.count
    }
    #[cfg(test)]
    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        backing::vector_bytes(&self.entries)
            .ok()?
            .checked_add(backing::vector_bytes(&self.slices).ok()?)
    }
    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.slices == other.slices
            && self.count == other.count
            && self.digest == other.digest
    }
}

/// The pending caller's local ceiling still reduces its original admission;
/// every full-tree observer additionally uses the same carried native pool.
pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &PhysicalRecoveryReadAllocation<'_>,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    expected_count: u64,
    expected_digest: [u8; 32],
    maximum_resident_bytes: u64,
) -> Result<ObservedReleaseHeads, Denial> {
    let mut resident = StoreRejoinResidentLedger::pending_head_walk_window(
        recovery_allocation,
        maximum_resident_bytes,
    )
    .map_err(Denial::Resident)?;
    observe_with_resident(
        discovery,
        window,
        root,
        format,
        recovery_allocation,
        expected_count,
        expected_digest,
        &mut resident,
    )
}

pub(super) fn observe_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &PhysicalRecoveryReadAllocation<'_>,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    expected_count: u64,
    expected_digest: [u8; 32],
    resident: &mut StoreRejoinResidentLedger,
) -> Result<ObservedReleaseHeads, Denial> {
    if discovery.store_identity() != recovery_allocation.store_identity()
        || window.store_identity() != discovery.store_identity()
        || window.recovery_byte_limit() != recovery_allocation.byte_limit()
    {
        return Err(Denial::RootBinding);
    }
    observation::observe_with_read(
        root,
        format,
        expected_count,
        expected_digest,
        recovery_allocation.byte_limit(),
        window,
        resident,
        |reference, remaining, storage| {
            let artifact = RecordArtifactFile::ReleaseCustodyHeadBlock {
                generation: reference.generation(),
                block: reference.block(),
            };
            discovery
                .read_record_artifact_with_storage(artifact, remaining, storage)
                .map_err(storage::read_denial)?
                .into_bytes()
                .ok_or(Denial::MissingFrame)
        },
    )
}
