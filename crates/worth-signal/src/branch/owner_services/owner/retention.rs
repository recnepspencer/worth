use crate::branch::retention::{
    SignalBranchAdmissionReservation, SignalBranchRetirementRetentionCounts,
};
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionBinding, SignalBranchRetentionLease, SignalBranchRetentionTerminalCounts,
};
use crate::state::SignalBranchId;

use super::super::owner_metadata::{
    SignalOwnerMetadataAuthorizationDenial, SignalOwnerRetentionAcquisitionDenial,
};
use super::super::{SignalOwnerOperationAdmission, SignalOwnerUnavailable};
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[allow(
        dead_code,
        reason = "Phase 4 basis operations consume this frozen owner seam"
    )]
    pub(in crate::branch::owner_services) fn acquire_admitted_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalBranchAdmissionLease, SignalBranchRetentionAcquisitionDenial> {
        Ok(self
            .reserve_admitted_retention(admission, branch_id, 1)?
            .into_one())
    }

    pub(in crate::branch::owner_services) fn reserve_admitted_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        lease_count: usize,
    ) -> Result<SignalBranchAdmissionReservation, SignalBranchRetentionAcquisitionDenial> {
        self.metadata
            .reserve_admitted_retention(
                admission,
                &self.retention,
                &self.counters,
                branch_id,
                lease_count,
            )
            .map_err(map_retention_acquisition_denial)
    }

    #[allow(
        dead_code,
        reason = "Phase 4 basis operations consume this frozen owner seam"
    )]
    pub(in crate::branch::owner_services) fn acquire_external_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        self.preflight_exact_external_retention(admission, basis)?;
        self.metadata
            .acquire_external_retention(admission, &self.retention, &self.counters, basis)
            .map_err(map_retention_acquisition_denial)
    }

    #[allow(
        dead_code,
        reason = "Phase 4 basis and lifecycle operations consume this seam"
    )]
    pub(in crate::branch::owner_services) fn retention_binding(
        &self,
    ) -> SignalBranchRetentionBinding {
        self.retention.binding()
    }

    #[allow(
        dead_code,
        reason = "Phase 4 lifecycle inspection consumes this frozen seam"
    )]
    pub(in crate::branch::owner_services) fn retention_terminal_counts(
        &self,
    ) -> SignalBranchRetentionTerminalCounts {
        self.retention.terminal_counts()
    }

    pub(super) fn retirement_retention_counts(
        &self,
        branch_id: SignalBranchId,
    ) -> SignalBranchRetirementRetentionCounts {
        self.counters.record_retention_registry_contact();
        self.retention.retirement_counts(branch_id)
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn admitted_retention_count(
        &self,
        branch_id: SignalBranchId,
    ) -> u32 {
        self.retention.admitted_count(branch_id)
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn admitted_or_reserved_retention_count(
        &self,
        branch_id: SignalBranchId,
    ) -> u32 {
        self.retention.admitted_or_reserved_count(branch_id)
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn retention_ledger_observation(
        &self,
    ) -> crate::branch::retention::SignalRetentionLedgerObservation {
        self.retention.test_observation()
    }
}

fn map_retention_acquisition_denial(
    denial: SignalOwnerRetentionAcquisitionDenial,
) -> SignalBranchRetentionAcquisitionDenial {
    match denial {
        SignalOwnerRetentionAcquisitionDenial::Metadata(
            SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable,
        ) => SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(SignalOwnerUnavailable),
        SignalOwnerRetentionAcquisitionDenial::Metadata(
            SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse
            | SignalOwnerMetadataAuthorizationDenial::OwnerReentry,
        ) => SignalBranchRetentionAcquisitionDenial::OwnerReentry,
        SignalOwnerRetentionAcquisitionDenial::Retention(denial) => denial,
    }
}
