//! Prepare error control storage before reading; settle only after C4 unwinds.

use std::{alloc::Layout, ffi::OsString, sync::Arc};

use worth_store_buffer_pool::OperationAllocationGrant;

use super::super::{
    PhysicalRecoveryObservationAllocationDenial as Denial, PhysicalRecoveryReadAllocation,
};
use super::{
    failure::{filename_capacity, FailureStorage, RawWalReadFailure},
    FundedRecoveryWalReadFailure,
};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;
use worth_store_physical_backend::RecoveryDiscoveryArtifact;

#[derive(Debug)]
pub(super) struct WalReadDiagnosticStorage {
    // Only the private preparation owner can mutate this unfinished state.
    // The public failure owner is constructed exclusively after Some is installed.
    pub(super) failure: Option<RawWalReadFailure>,
    pub(super) backing: OperationAllocationGrant,
}

pub(super) struct PreparedWalReadDiagnostic {
    storage: Arc<WalReadDiagnosticStorage>,
}

impl PreparedWalReadDiagnostic {
    pub(super) fn prepare(
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<Self, FundedRecoveryWalReadFailure> {
        let requested = storage_bytes();
        let backing = window
            .reserve_owned(requested as u64)
            .map_err(|cause| {
                FundedRecoveryWalReadFailure::inline(requested, Denial::Residency(cause))
            })?
            .expect("diagnostic control storage has positive size");
        Ok(Self {
            storage: Arc::new(WalReadDiagnosticStorage {
                failure: None,
                backing,
            }),
        })
    }

    /// C4 calls this only after its previous successful context has been disposed.
    pub(super) fn allocate_context(
        &mut self,
        window: &PhysicalRecoveryReadAllocation<'_>,
        requested: usize,
    ) -> Result<OsString, Denial> {
        let required = storage_bytes().checked_add(requested).ok_or_else(|| {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                admitted: window.recovery_byte_limit(),
            })
        })?;
        let storage = Arc::get_mut(&mut self.storage).expect("preparation has no observers");
        storage
            .backing
            .try_resize(required as u64)
            .map_err(|cause| Denial::Residency(window.map_allocation_denial(cause)))?;
        let mut context = OsString::new();
        context.try_reserve_exact(requested).map_err(|cause| {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::Allocation {
                requested: requested as u64,
                cause,
            })
        })?;
        let actual = context.capacity();
        if actual != requested {
            return Err(Denial::AllocatorExceededReservation {
                requested: requested as u64,
                actual: actual as u64,
            });
        }
        Ok(context)
    }

    pub(super) fn finish(mut self, failure: RawWalReadFailure) -> FundedRecoveryWalReadFailure {
        // Refusing the context itself must never allocate or copy its name.
        // The already prepared control allocation is no longer needed either.
        if let RawWalReadFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            requested,
            cause,
            ..
        } = failure
        {
            return FundedRecoveryWalReadFailure::inline(requested, cause);
        }
        // Source roster/payload backing and C4 scratch have already been disposed.
        let retained = storage_bytes()
            .checked_add(filename_capacity(&failure))
            .expect("previously admitted filename capacity");
        let storage = Arc::get_mut(&mut self.storage).expect("preparation has no observers");
        assert!(
            retained as u64 <= storage.backing.bytes(),
            "failure may retain only pre-funded context"
        );
        storage.failure = Some(failure);
        storage
            .backing
            .try_resize(retained as u64)
            .expect("private diagnostic backing has no named live uses");
        FundedRecoveryWalReadFailure {
            storage: FailureStorage::Shared(self.storage),
        }
    }
}

pub(super) fn storage_bytes() -> usize {
    // ArcInner's two atomic counters precede the aligned diagnostic data.
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<WalReadDiagnosticStorage>())
        .expect("fixed diagnostic layout fits")
        .0
        .pad_to_align()
        .size()
}
