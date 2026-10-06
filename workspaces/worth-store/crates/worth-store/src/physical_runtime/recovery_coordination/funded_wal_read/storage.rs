//! One Store-owned read port keeps listing, result and diagnostic lifetimes distinct.

use std::ffi::OsString;

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    ArtifactTreeDirectoryEntry, ArtifactTreeListingAllocator, ArtifactTreeListingStorageChange,
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator, RecoveryWalListingAllocationMode, RecoveryWalReadStorage,
};

use super::super::{
    PhysicalRecoveryObservationAllocationDenial as Denial, PhysicalRecoveryReadAllocation,
};
use super::{
    diagnostic_backing::PreparedWalReadDiagnostic, failure::RawWalReadFailure,
    listing_backing::NativeWalListingBacking, FundedRecoveryWalObservations,
    FundedRecoveryWalReadFailure, ObservedWalArtifact, WalObservationBacking,
};

pub(super) struct NativeWalReadStorage<'window, 'coordination> {
    result: WalObservationBacking<'window, 'coordination>,
    listing: NativeWalListingBacking,
    diagnostic: PreparedWalReadDiagnostic,
}

impl<'window, 'coordination> NativeWalReadStorage<'window, 'coordination> {
    pub(super) fn prepare(
        window: &'window PhysicalRecoveryReadAllocation<'coordination>,
    ) -> Result<Self, FundedRecoveryWalReadFailure> {
        let diagnostic = PreparedWalReadDiagnostic::prepare(window)?;
        Ok(Self {
            result: WalObservationBacking {
                window,
                backing: None,
                retained: 0,
            },
            listing: NativeWalListingBacking::default(),
            diagnostic,
        })
    }

    pub(super) fn finish_observations(
        self,
        artifacts: Vec<ObservedWalArtifact>,
    ) -> FundedRecoveryWalObservations {
        let Self {
            result,
            listing,
            diagnostic,
        } = self;
        let names = listing.into_names(&artifacts);
        drop(diagnostic);
        FundedRecoveryWalObservations {
            artifacts,
            backing: result.backing,
            names,
        }
    }

    pub(super) fn finish_failure(self, failure: RawWalReadFailure) -> FundedRecoveryWalReadFailure {
        let Self {
            result,
            listing,
            diagnostic,
        } = self;
        // All C4 provider/listing/source buffers have unwound before releasing backing.
        drop(result);
        drop(listing);
        diagnostic.finish(failure)
    }
}

impl ArtifactTreeStorageAllocator for NativeWalReadStorage<'_, '_> {
    type Denial = Denial;
}

impl ArtifactTreeListingAllocator for NativeWalReadStorage<'_, '_> {
    fn listing_storage_change(
        &mut self,
        change: ArtifactTreeListingStorageChange,
    ) -> Result<(), Denial> {
        self.listing.change(self.result.window, change)
    }

    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, Denial> {
        self.listing.allocate_roster(count)
    }
}

impl ArtifactTreePathAllocator for NativeWalReadStorage<'_, '_> {
    type PathBacking = Option<OperationAllocationGrant>;

    fn admit_path_backing(
        &mut self,
        boundary: ArtifactTreePathAllocationBoundary,
        requested_bytes: u64,
    ) -> Result<Self::PathBacking, Denial> {
        self.result
            .window
            .reserve_owned(requested_bytes)
            .map_err(|cause| Denial::PathResidency { boundary, cause })
    }
}

impl ArtifactTreeReadAllocator for NativeWalReadStorage<'_, '_> {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, Denial> {
        self.result.allocate_payload(length)
    }
}

impl RecoveryWalReadStorage for NativeWalReadStorage<'_, '_> {
    fn listing_allocation_mode(&self) -> RecoveryWalListingAllocationMode {
        RecoveryWalListingAllocationMode::AdmissionCallbacks
    }

    fn allocate_wal_roster(&mut self, count: usize) -> Result<Vec<ObservedWalArtifact>, Denial> {
        self.result.allocate_vector(count)
    }

    fn allocate_context(&mut self, count: usize) -> Result<OsString, Denial> {
        self.diagnostic.allocate_context(self.result.window, count)
    }
}
