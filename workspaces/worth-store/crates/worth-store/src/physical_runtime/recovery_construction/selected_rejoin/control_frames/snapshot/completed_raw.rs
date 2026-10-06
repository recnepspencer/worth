//! Native custody of completed-history routing and control slice storage.
//! The walk prepares each allocation before it occurs; the final grant moves
//! with the exact retained slice vector into Serving.

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalResidencyIncarnation};
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{SelectedArtifactSlice, SelectedMediaRejoinDenial as Denial};
use crate::physical_runtime::{
    record_serving::RecordBootstrapDenial, LifecycleGeneration, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial as ResidentDenial, PhysicalScopedAllocationFailure,
};

pub(in crate::physical_runtime::recovery_construction) struct FundedCompletedHistoricalRawSlices {
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    origin: LifecycleGeneration,
    backing: Option<OperationAllocationGrant>,
    retained: u64,
}

impl FundedCompletedHistoricalRawSlices {
    pub(in crate::physical_runtime::recovery_construction) fn new(
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<Self, Denial> {
        Ok(Self {
            store: window.store_identity(),
            pool: window.pool_identity(),
            origin: window
                .recovery_origin_generation()
                .ok_or(Denial::Resident(ResidentDenial::MissingResidentAdmission))?,
            backing: None,
            retained: 0,
        })
    }

    pub(in crate::physical_runtime::recovery_construction) fn matching_owner(
        &self,
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> bool {
        self.store == window.store_identity()
            && self.pool == window.pool_identity()
            && Some(self.origin) == window.recovery_origin_generation()
    }

    pub(in crate::physical_runtime::recovery_construction) fn prepare(
        &mut self,
        window: &PhysicalRecoveryReadAllocation<'_>,
        required: u64,
    ) -> Result<(), Denial> {
        if !self.matching_owner(window) {
            return Err(Denial::RootBinding);
        }
        if required <= self.charged_bytes() {
            return Ok(());
        }
        match &mut self.backing {
            Some(grant) => grant.try_resize(required).map_err(|cause| {
                Denial::Resident(ResidentDenial::OperationAllocation(
                    PhysicalScopedAllocationFailure::from_denial(cause, self.origin),
                ))
            }),
            None => {
                self.backing = window.reserve_owned(required).map_err(Denial::Resident)?;
                Ok(())
            }
        }
    }

    /// The caller has already disposed every temporary allocation above the
    /// retained total. Shrinking never creates a new native admission.
    pub(in crate::physical_runtime::recovery_construction) fn settle(
        &mut self,
        retained: u64,
    ) -> Result<(), Denial> {
        if retained > self.retained || retained > self.charged_bytes() {
            return Err(Denial::BoundExceeded);
        }
        if let Some(grant) = &mut self.backing {
            grant.try_resize(retained).map_err(|cause| {
                Denial::Resident(ResidentDenial::OperationAllocation(
                    PhysicalScopedAllocationFailure::from_denial(cause, self.origin),
                ))
            })?;
        }
        self.retained = retained;
        Ok(())
    }

    pub(in crate::physical_runtime::recovery_construction) fn retain(
        &mut self,
        bytes: u64,
    ) -> Result<(), Denial> {
        let next = self
            .retained
            .checked_add(bytes)
            .ok_or(Denial::BoundExceeded)?;
        if next > self.charged_bytes() {
            return Err(Denial::BoundExceeded);
        }
        self.retained = next;
        Ok(())
    }

    pub(in crate::physical_runtime::recovery_construction) fn retained_bytes(&self) -> u64 {
        self.retained
    }

    pub(in crate::physical_runtime::recovery_construction) fn charged_bytes(&self) -> u64 {
        self.backing
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
    }

    pub(super) fn verify_serving_media(
        &self,
        slices: &[SelectedArtifactSlice],
        media: &QualifiedFilesystemMedia,
        format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, RecordBootstrapDenial> {
        if !self.matching_owner(window) {
            return Err(RecordBootstrapDenial::RecoveredHeadWitnessOwnerMismatch);
        }
        let bytes = slices
            .iter()
            .try_fold(0_u64, |sum, slice| sum.checked_add(u64::from(slice.length)))
            .ok_or(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
        if slices.is_empty() {
            return Ok(true);
        }
        let mut observation = media
            .bounded_record_observation(slices.len() as u64, bytes)
            .map_err(RecordBootstrapDenial::RecoveredHeadObservationUnavailable)?;
        for slice in slices {
            if !slice.matches_funded_serving_media(&mut observation, format, window)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
