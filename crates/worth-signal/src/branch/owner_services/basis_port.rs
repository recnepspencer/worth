use std::sync::{Arc, Weak};

use crate::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference,
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisDescriptor,
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetainedReadmissionDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionLease, SignalBranchRetentionOwnerRelationship,
    SignalBranchRetentionReleaseDenial, SignalBranchRetentionReleaseOutcome,
};

use super::owner::basis::map_basis_registry_denial;
use super::{
    SignalOwner, SignalOwnerLifecycleObservation, SignalOwnerServiceCostSnapshot,
    SignalOwnerUnavailable,
};

pub(super) mod denial_mapping;
pub(super) mod descriptor_validation;
#[cfg(test)]
mod tests;

use denial_mapping::{
    map_basis_admission_denial, map_managed_observation_admission_denial,
    map_managed_readmission_admission_denial, map_observation_readmission_denial,
    map_observation_retention_denial, map_readmission_retention_denial,
    map_release_admission_denial, map_retained_admission_denial, map_retained_retention_denial,
    map_retention_admission_denial,
};
use descriptor_validation::compare_descriptor_with_observation;

/// Concrete weak service for exact Signal basis observation and retention.
pub struct SignalBranchBasisPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Weak<SignalOwner<D, I, T>>,
    diagnostic_owner_runtime_instance_id: u64,
}

impl<D, I, T> Clone for SignalBranchBasisPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            diagnostic_owner_runtime_instance_id: self.diagnostic_owner_runtime_instance_id,
        }
    }
}

impl<D, I, T> SignalBranchBasisPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        owner: Weak<SignalOwner<D, I, T>>,
        diagnostic_owner_runtime_instance_id: u64,
    ) -> Self {
        Self {
            owner,
            diagnostic_owner_runtime_instance_id,
        }
    }

    pub(crate) fn diagnostic_owner_runtime_instance_id(&self) -> u64 {
        self.diagnostic_owner_runtime_instance_id
    }

    pub fn issue_managed_branch_reference(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable)?;
        owner.issue_managed_branch_reference(basis)
    }

    pub fn observe_current(
        &self,
        reference: &ManagedSignalBranchReference,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(SignalBranchBasisObservationDenial::OwnerUnavailable)?;
        let branch_id = reference.branch_id();
        let (admission, cell) = owner
            .admit_managed_branch_reference(reference)
            .map_err(map_managed_observation_admission_denial)?;
        let observation = cell.observe_exact(&admission)?;
        owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                branch_id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, branch_id),
            )
            .map_err(|denial| {
                map_observation_retention_denial(&owner, &admission, denial, branch_id)
            })
    }

    pub fn readmit_exact(
        &self,
        reference: &ManagedSignalBranchReference,
        descriptor: &SignalBranchBasisDescriptor,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchBasisReadmissionDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(SignalBranchBasisReadmissionDenial::OwnerUnavailable)?;
        let branch_id = reference.branch_id();
        let (admission, cell) = owner
            .admit_managed_branch_reference(reference)
            .map_err(map_managed_readmission_admission_denial)?;
        owner.validate_managed_basis_descriptor(descriptor, branch_id)?;
        let observation = cell.observe_exact(&admission).map_err(|denial| {
            map_observation_readmission_denial(&owner, &admission, denial, branch_id)
        })?;
        compare_descriptor_with_observation(descriptor, &observation)?;
        owner
            .admit_canonical_basis_with_retention(
                &admission,
                observation,
                branch_id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, branch_id),
            )
            .map_err(|denial| {
                map_readmission_retention_denial(&owner, &admission, denial, branch_id)
            })
    }

    pub fn compare_current_exact(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<(), SignalBranchBasisReadmissionDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(SignalBranchBasisReadmissionDenial::OwnerUnavailable)?;
        match basis.owner_identity_relationship(&owner.retention_binding()) {
            crate::branch::SignalBranchRetentionOwnerRelationship::SameOwner => {}
            crate::branch::SignalBranchRetentionOwnerRelationship::DifferentOwner => {
                let descriptor_graph_instance_id = basis
                    .descriptor()
                    .observation()
                    .target()
                    .as_basis()
                    .map_or_else(
                        || "<non-basis>".to_owned(),
                        |target| target.graph_instance_id().to_owned(),
                    );
                return Err(SignalBranchBasisReadmissionDenial::OwnerMismatch {
                    descriptor_graph_instance_id,
                    runtime_graph_instance_id: owner.runtime_instance_id().to_string(),
                });
            }
            crate::branch::SignalBranchRetentionOwnerRelationship::OwnerLost => {
                return Err(SignalBranchBasisReadmissionDenial::OwnerUnavailable(
                    SignalOwnerUnavailable,
                ));
            }
        }
        let branch_id = basis.owner_branch_id();
        owner.validate_managed_basis_descriptor(basis.descriptor(), branch_id)?;
        let admission = owner.admit().map_err(map_basis_admission_denial)?;
        let cell = owner.lookup_cell(&admission, branch_id).map_err(|denial| {
            map_observation_readmission_denial(
                &owner,
                &admission,
                map_basis_registry_denial(denial, branch_id),
                branch_id,
            )
        })?;
        let cell_incarnation = cell.incarnation().get();
        let observation = cell.observe_exact(&admission).map_err(|denial| {
            map_observation_readmission_denial(&owner, &admission, denial, branch_id)
        })?;
        compare_descriptor_with_observation(basis.descriptor(), &observation)?;
        if !owner.is_current_canonical_basis(basis, branch_id, cell_incarnation, &observation) {
            return Err(SignalBranchBasisReadmissionDenial::LifecycleMismatch);
        }
        Ok(())
    }

    pub fn readmit_retained_exact(
        &self,
        descriptor: &SignalBranchBasisDescriptor,
        lease: &SignalBranchRetentionLease,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchRetainedReadmissionDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(SignalBranchRetainedReadmissionDenial::OwnerUnavailable)?;
        match lease.owner_relationship(&owner.retention_binding()) {
            SignalBranchRetentionOwnerRelationship::DifferentOwner => {
                return Err(SignalBranchRetainedReadmissionDenial::ForeignRetention)
            }
            SignalBranchRetentionOwnerRelationship::OwnerLost => {
                let _admission = owner.admit().map_err(map_retained_admission_denial)?;
                return Err(SignalBranchRetainedReadmissionDenial::UnavailableRetainedTarget);
            }
            SignalBranchRetentionOwnerRelationship::SameOwner => {}
        }
        let admission = owner.admit().map_err(map_retained_admission_denial)?;
        owner.preflight_retained_readmission(&admission, descriptor, lease)?;
        let branch_id = descriptor.branch_id();
        let cell = owner
            .lookup_cell(&admission, branch_id)
            .map_err(|_| SignalBranchRetainedReadmissionDenial::UnavailableRetainedTarget)?;
        owner
            .admit_canonical_basis_with_retention(
                &admission,
                descriptor.observation().clone(),
                branch_id,
                cell.incarnation().get(),
                || owner.acquire_admitted_retention(&admission, branch_id),
            )
            .map_err(map_retained_retention_denial)
    }

    pub fn retain_exact(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        let owner = self
            .upgrade_owner()
            .map_err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable)?;
        let admission = owner.admit().map_err(map_retention_admission_denial)?;
        owner.acquire_external_retention(&admission, basis)
    }

    pub fn release_exact(
        &self,
        lease: SignalBranchRetentionLease,
    ) -> SignalBranchRetentionReleaseOutcome {
        let owner = match self.upgrade_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: SignalBranchRetentionReleaseDenial::OwnerUnavailable(unavailable),
                }
            }
        };
        match lease.owner_relationship(&owner.retention_binding()) {
            SignalBranchRetentionOwnerRelationship::DifferentOwner => {
                SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: SignalBranchRetentionReleaseDenial::ForeignRuntime,
                }
            }
            SignalBranchRetentionOwnerRelationship::OwnerLost => match owner.admit() {
                Ok(_admission) => SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: SignalBranchRetentionReleaseDenial::OwnerUnavailable(
                        SignalOwnerUnavailable,
                    ),
                },
                Err(denial) => SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: map_release_admission_denial(denial),
                },
            },
            SignalBranchRetentionOwnerRelationship::SameOwner => match owner.admit() {
                Ok(_admission) => SignalBranchRetentionReleaseOutcome::Released(lease.release()),
                Err(denial) => SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: map_release_admission_denial(denial),
                },
            },
        }
    }

    pub fn owner_lifecycle_observation(&self) -> SignalOwnerLifecycleObservation {
        self.upgrade_owner()
            .map_or(SignalOwnerLifecycleObservation::Closed, |owner| {
                owner.lifecycle_observation()
            })
    }

    pub fn owner_service_cost_snapshot(
        &self,
    ) -> Result<SignalOwnerServiceCostSnapshot, SignalOwnerUnavailable> {
        let owner = self.upgrade_owner()?;
        match owner.lifecycle_observation() {
            SignalOwnerLifecycleObservation::Open => Ok(owner.cost_snapshot()),
            SignalOwnerLifecycleObservation::Closing | SignalOwnerLifecycleObservation::Closed => {
                Err(SignalOwnerUnavailable)
            }
        }
    }

    pub(super) fn upgrade_owner(
        &self,
    ) -> Result<Arc<SignalOwner<D, I, T>>, SignalOwnerUnavailable> {
        SignalOwner::upgrade(&self.owner)
    }
}
