use worth_foundational::FoundationalBranchTargetBasis;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisDescriptor, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionLease,
};
use crate::state::SignalBranchId;

use super::super::accounting::obligation_count;
use super::{SignalBranchRetentionRegistry, SignalRetainedTargetKey};

pub(crate) struct SignalExternalRetentionAcquisition {
    descriptor: SignalBranchBasisDescriptor,
    branch_id: SignalBranchId,
    target: SignalRetainedTargetKey,
}

impl SignalExternalRetentionAcquisition {
    pub(crate) const fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalBranchRetirementRetentionCounts {
    pub(crate) admitted_or_reserved: u32,
    pub(crate) external: u32,
}

impl SignalBranchRetentionRegistry {
    pub(crate) fn acquire_external(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        let acquisition = self.prepare_external_acquisition(basis)?;
        self.commit_external_acquisition(acquisition)
    }

    pub(crate) fn prepare_external_acquisition(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalExternalRetentionAcquisition, SignalBranchRetentionAcquisitionDenial> {
        let descriptor = basis.descriptor();
        let Some(target) = descriptor.observation().target().as_basis() else {
            return Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis);
        };
        if target.graph_instance_id() != self.owner.runtime_instance_id.to_string() {
            return Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis);
        }
        let branch_id = descriptor.owner_branch_id();
        let target = SignalRetainedTargetKey(target.canonical_encoding());
        let acquisition = SignalExternalRetentionAcquisition {
            descriptor: descriptor.clone(),
            branch_id,
            target,
        };
        Ok(acquisition)
    }

    /// Commit one exact immutable obligation while the owner kernel holds its
    /// branch-specific metadata acquisition fence.
    pub(crate) fn commit_external_acquisition(
        &self,
        acquisition: SignalExternalRetentionAcquisition,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        let lease_id = self
            .ledger
            .retain_exact_target(acquisition.branch_id, acquisition.target)?;
        Ok(SignalBranchRetentionLease::owner_issued(
            acquisition.descriptor,
            self.binding(),
            lease_id,
        ))
    }

    pub(crate) fn retirement_counts(
        &self,
        branch_id: SignalBranchId,
    ) -> SignalBranchRetirementRetentionCounts {
        let state = self.ledger.lock();
        SignalBranchRetirementRetentionCounts {
            admitted_or_reserved: obligation_count(&state.admitted_count_by_branch, &branch_id)
                .saturating_add(obligation_count(
                    &state.reserved_admitted_count_by_branch,
                    &branch_id,
                )),
            external: obligation_count(&state.external_count_by_branch, &branch_id),
        }
    }
}
