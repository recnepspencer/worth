//! Qualified address/open backing is temporary; actual payload backing moves
//! with the observed buffer rather than the borrowed read window.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator, ObservedRecoveryArtifact,
};

use super::{
    FundedRecoveryObservation, ObservationBacking,
    PhysicalRecoveryObservationAllocationDenial as Denial, PhysicalRecoveryReadAllocation,
};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

pub(super) struct NativeObservationStorage<'window, 'coordination> {
    window: &'window PhysicalRecoveryReadAllocation<'coordination>,
    payload: Option<OperationAllocationGrant>,
}

impl<'window, 'coordination> NativeObservationStorage<'window, 'coordination> {
    pub(super) fn new(window: &'window PhysicalRecoveryReadAllocation<'coordination>) -> Self {
        Self {
            window,
            payload: None,
        }
    }

    pub(super) fn finish(self, observed: ObservedRecoveryArtifact) -> FundedRecoveryObservation {
        let backing = match self.payload {
            Some(grant) if observed.bytes().is_some() => ObservationBacking::Reserved(grant),
            // An absent read retains no payload, including an absent range
            // whose prepared buffer was disposed inside C4 before returning.
            _ => ObservationBacking::Inline,
        };
        FundedRecoveryObservation { observed, backing }
    }
}

impl ArtifactTreeStorageAllocator for NativeObservationStorage<'_, '_> {
    type Denial = Denial;
}

impl ArtifactTreePathAllocator for NativeObservationStorage<'_, '_> {
    type PathBacking = Option<OperationAllocationGrant>;

    fn admit_path_backing(
        &mut self,
        boundary: ArtifactTreePathAllocationBoundary,
        requested_bytes: u64,
    ) -> Result<Self::PathBacking, Denial> {
        self.window
            .reserve_owned(requested_bytes)
            .map_err(|cause| Denial::PathResidency { boundary, cause })
    }
}

impl ArtifactTreeReadAllocator for NativeObservationStorage<'_, '_> {
    fn allocate_read_buffer(&mut self, requested: usize) -> Result<Vec<u8>, Denial> {
        let bytes = u64::try_from(requested).map_err(|_| {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                admitted: self.window.recovery_byte_limit(),
            })
        })?;
        // C4 opens one selected file and invokes this port once. Retained
        // observations each carry their own grant in the same native owner.
        assert!(
            self.payload.is_none(),
            "one buffer per addressed observation"
        );
        self.payload = self
            .window
            .reserve_owned(bytes)
            .map_err(Denial::Residency)?;
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(requested).map_err(|cause| {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::Allocation {
                requested: bytes,
                cause,
            })
        })?;
        if buffer.capacity() as u64 != bytes {
            return Err(Denial::AllocatorExceededReservation {
                requested: bytes,
                actual: buffer.capacity() as u64,
            });
        }
        buffer.resize(requested, 0);
        Ok(buffer)
    }
}
