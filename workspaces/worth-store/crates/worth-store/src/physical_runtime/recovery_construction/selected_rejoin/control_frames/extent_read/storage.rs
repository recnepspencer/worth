//! Allocation/read port for the one selected-extent membership predicate.

use worth_store_physical_backend::{
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, ReadGrant, UnchargedRead,
};
use worth_store_physical_format::DurableExtentRecordPlacement;

use super::{Denial, SelectedArtifactSlice};
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::{
    discovery_allocation_denial, StoreRejoinResidentLedger,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) trait ExtentReadStorage {
    fn grow_slices(&mut self, slices: &mut Vec<SelectedArtifactSlice>) -> Result<(), Denial>;
    fn reserve_payload(&mut self, count: usize) -> Result<Vec<u8>, Denial>;
    fn read_manifest(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
    ) -> Result<ObservedRecoveryArtifact, Denial>;
    fn read_chunk(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        relative: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial>;
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial>;
}

impl ExtentReadStorage for () {
    fn grow_slices(&mut self, _slices: &mut Vec<SelectedArtifactSlice>) -> Result<(), Denial> {
        Ok(())
    }
    fn reserve_payload(&mut self, count: usize) -> Result<Vec<u8>, Denial> {
        Ok(Vec::with_capacity(count))
    }
    fn read_manifest(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_extent_manifest(placement.arena_range(), ReadGrant::ceiling_only())
            .observed()
            .map_err(Denial::Discovery)
    }
    fn read_chunk(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        relative: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_extent_range(
                placement.arena_range(),
                relative,
                length,
                ReadGrant::ceiling_only(),
            )
            .observed()
            .map_err(Denial::Discovery)
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        drop(frame);
        Ok(())
    }
}

impl ExtentReadStorage for StoreRejoinResidentLedger {
    fn grow_slices(&mut self, slices: &mut Vec<SelectedArtifactSlice>) -> Result<(), Denial> {
        self.grow_vec_geometrically(slices, 1)
            .map_err(Denial::Resident)
    }
    fn reserve_payload(&mut self, count: usize) -> Result<Vec<u8>, Denial> {
        self.reserve_vec(count).map_err(Denial::Resident)
    }
    fn read_manifest(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_extent_manifest_with_allocator(
                placement.arena_range(),
                ReadGrant::ceiling_only(),
                |length| self.reserve_bytes(length),
            )
            .observed()
            .map_err(discovery_allocation_denial)
    }
    fn read_chunk(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        placement: DurableExtentRecordPlacement,
        relative: u64,
        length: u32,
    ) -> Result<ObservedRecoveryArtifact, Denial> {
        discovery
            .read_extent_range_with_allocator(
                placement.arena_range(),
                relative,
                length,
                ReadGrant::ceiling_only(),
                |length| self.reserve_bytes(length),
            )
            .observed()
            .map_err(discovery_allocation_denial)
    }
    fn discard_frame(&mut self, frame: ObservedRecoveryArtifact) -> Result<(), Denial> {
        let bytes = frame.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
        drop(frame);
        self.release(bytes);
        Ok(())
    }
}
