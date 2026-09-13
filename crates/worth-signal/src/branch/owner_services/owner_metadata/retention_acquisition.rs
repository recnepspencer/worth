use crate::branch::retention::{
    SignalBranchAdmissionReservation, SignalExternalRetentionAcquisition,
};
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionLease,
    SignalBranchRetentionRegistry,
};
use crate::state::SignalBranchId;

use super::super::{SignalOwnerOperationAdmission, SignalOwnerServiceCounters};
use super::{SignalOwnerMetadata, SignalOwnerMetadataAuthorizationDenial};

pub(in crate::branch::owner_services) enum SignalOwnerRetentionAcquisitionDenial {
    Metadata(SignalOwnerMetadataAuthorizationDenial),
    Retention(SignalBranchRetentionAcquisitionDenial),
}

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_admitted_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        retention: &SignalBranchRetentionRegistry,
        counters: &SignalOwnerServiceCounters,
        branch_id: SignalBranchId,
        lease_count: usize,
    ) -> Result<SignalBranchAdmissionReservation, SignalOwnerRetentionAcquisitionDenial> {
        let _admission_hold = self
            .authorize(admission)
            .map_err(SignalOwnerRetentionAcquisitionDenial::Metadata)?;
        let metadata = self.lock();
        if !metadata.branch_accepts_retention_acquisition(branch_id) {
            return Err(SignalOwnerRetentionAcquisitionDenial::Retention(
                SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id },
            ));
        }
        counters.record_retention_registry_contact();
        retention
            .reserve_admitted(branch_id, lease_count)
            .map_err(SignalOwnerRetentionAcquisitionDenial::Retention)
    }

    pub(in crate::branch::owner_services) fn acquire_external_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        retention: &SignalBranchRetentionRegistry,
        counters: &SignalOwnerServiceCounters,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalOwnerRetentionAcquisitionDenial> {
        let _admission_hold = self
            .authorize(admission)
            .map_err(SignalOwnerRetentionAcquisitionDenial::Metadata)?;
        let acquisition: SignalExternalRetentionAcquisition = retention
            .prepare_external_acquisition(basis)
            .map_err(SignalOwnerRetentionAcquisitionDenial::Retention)?;
        let branch_id = acquisition.branch_id();
        let metadata = self.lock();
        if !metadata.branch_accepts_retention_acquisition(branch_id) {
            return Err(SignalOwnerRetentionAcquisitionDenial::Retention(
                SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id },
            ));
        }
        counters.record_retention_registry_contact();
        retention
            .commit_external_acquisition(acquisition)
            .map_err(SignalOwnerRetentionAcquisitionDenial::Retention)
    }
}
