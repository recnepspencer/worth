use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::branch::retention::SignalBranchRetentionOwnerRelationship;
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisDescriptor, SignalBranchBasisLifecyclePosture,
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetainedReadmissionDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionLease, SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
};
use crate::state::SignalBranchId;

use super::super::owner_metadata::SignalOwnerMetadataAuthorizationDenial;
use super::super::{
    SignalOwnerLifecycleObservation, SignalOwnerOperationAdmission, SignalOwnerUnavailable,
};
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn validate_managed_basis_descriptor(
        &self,
        descriptor: &SignalBranchBasisDescriptor,
        expected_branch_id: SignalBranchId,
    ) -> Result<(), SignalBranchBasisReadmissionDenial> {
        if descriptor.schema_version() != SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION {
            return Err(
                SignalBranchBasisReadmissionDenial::UnsupportedDescriptorVersion {
                    observed: descriptor.schema_version(),
                    supported: SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
                },
            );
        }
        if descriptor.lifecycle_posture() != SignalBranchBasisLifecyclePosture::Live {
            return Err(SignalBranchBasisReadmissionDenial::LifecycleMismatch);
        }
        if descriptor.branch_id() != expected_branch_id {
            return Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch {
                axes: vec![FoundationalBranchReferenceMismatchAxis::BranchIdentity],
            });
        }
        self.validate_basis_descriptor_affinity(descriptor)
            .map_err(map_managed_basis_affinity_denial)
    }

    pub(in crate::branch::owner_services) fn validate_canonical_basis_reuse(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<(), SignalBranchRetentionAcquisitionDenial> {
        if self.lifecycle_observation() != SignalOwnerLifecycleObservation::Open {
            return Err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(
                SignalOwnerUnavailable,
            ));
        }
        match self
            .metadata
            .branch_accepts_retention_acquisition(admission, branch_id)
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id }),
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable) => Err(
                SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(SignalOwnerUnavailable),
            ),
            Err(
                SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse
                | SignalOwnerMetadataAuthorizationDenial::OwnerReentry,
            ) => Err(SignalBranchRetentionAcquisitionDenial::OwnerReentry),
        }
    }

    pub(in crate::branch::owner_services) fn validate_retained_basis_descriptor(
        &self,
        descriptor: &SignalBranchBasisDescriptor,
        lease: &SignalBranchRetentionLease,
    ) -> Result<(), SignalBranchRetainedReadmissionDenial> {
        if !lease.retains_live_obligation() {
            return Err(SignalBranchRetainedReadmissionDenial::UnavailableRetainedTarget);
        }
        if descriptor != lease.descriptor() {
            return Err(SignalBranchRetainedReadmissionDenial::DescriptorMismatch);
        }
        if descriptor.schema_version() != SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION {
            return Err(
                SignalBranchRetainedReadmissionDenial::UnsupportedDescriptorVersion {
                    observed: descriptor.schema_version(),
                    supported: SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
                },
            );
        }
        if descriptor.lifecycle_posture() != SignalBranchBasisLifecyclePosture::Live {
            return Err(SignalBranchRetainedReadmissionDenial::LifecycleMismatch);
        }
        self.validate_basis_descriptor_affinity(descriptor)
            .map_err(map_retained_basis_affinity_denial)
    }

    pub(in crate::branch::owner_services) fn basis_has_owner_affinity(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> bool {
        basis.owner_identity_relationship(&self.retention.binding())
            == SignalBranchRetentionOwnerRelationship::SameOwner
            && self
                .validate_basis_descriptor_affinity(basis.descriptor())
                .is_ok()
    }

    pub(super) fn validate_external_retention_basis(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<(), SignalBranchRetentionAcquisitionDenial> {
        if basis.owner_identity_relationship(&self.retention.binding())
            != SignalBranchRetentionOwnerRelationship::SameOwner
        {
            return Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis);
        }
        self.validate_basis_descriptor_affinity(basis.descriptor())
            .map_err(map_retention_basis_affinity_denial)
    }

    pub(in crate::branch::owner_services) fn resolve_observation_retirement_denial(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> SignalBranchBasisObservationDenial {
        match self.retirement_posture(admission, branch_id) {
            Ok(SignalOwnerBasisRetirementPosture::InProgress) => {
                SignalBranchBasisObservationDenial::RetirementInProgress { branch_id }
            }
            Ok(SignalOwnerBasisRetirementPosture::Retired) => {
                SignalBranchBasisObservationDenial::RetiredBranch { branch_id }
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable) => {
                SignalBranchBasisObservationDenial::OwnerUnavailable(SignalOwnerUnavailable)
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse) => {
                SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id }
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerReentry) => {
                SignalBranchBasisObservationDenial::OwnerReentry
            }
        }
    }

    pub(in crate::branch::owner_services) fn resolve_readmission_retirement_denial(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> SignalBranchBasisReadmissionDenial {
        match self.retirement_posture(admission, branch_id) {
            Ok(SignalOwnerBasisRetirementPosture::InProgress) => {
                SignalBranchBasisReadmissionDenial::RetirementInProgress { branch_id }
            }
            Ok(SignalOwnerBasisRetirementPosture::Retired) => {
                SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id }
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable) => {
                SignalBranchBasisReadmissionDenial::OwnerUnavailable(SignalOwnerUnavailable)
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse) => {
                SignalBranchBasisReadmissionDenial::OwnerCellMisuse { branch_id }
            }
            Err(SignalOwnerMetadataAuthorizationDenial::OwnerReentry) => {
                SignalBranchBasisReadmissionDenial::OwnerReentry
            }
        }
    }

    fn retirement_posture(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalOwnerBasisRetirementPosture, SignalOwnerMetadataAuthorizationDenial> {
        Ok(
            if self
                .metadata
                .retirement_receipt(admission, branch_id)?
                .is_some()
            {
                SignalOwnerBasisRetirementPosture::Retired
            } else {
                SignalOwnerBasisRetirementPosture::InProgress
            },
        )
    }

    fn validate_basis_descriptor_affinity(
        &self,
        descriptor: &SignalBranchBasisDescriptor,
    ) -> Result<(), SignalOwnerBasisAffinityDenial> {
        let target = descriptor
            .observation()
            .target()
            .as_basis()
            .ok_or(SignalOwnerBasisAffinityDenial::TargetBasis)?;
        let runtime_graph_instance_id = self.runtime_instance_id().to_string();
        if target.graph_instance_id() != runtime_graph_instance_id {
            return Err(SignalOwnerBasisAffinityDenial::Owner {
                descriptor_graph_instance_id: target.graph_instance_id().to_owned(),
                runtime_graph_instance_id,
            });
        }
        if target.definition_basis() != self.definition_basis() {
            return Err(SignalOwnerBasisAffinityDenial::Definition {
                descriptor_definition_basis: target.definition_basis(),
                runtime_definition_basis: self.definition_basis(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalOwnerBasisRetirementPosture {
    InProgress,
    Retired,
}

enum SignalOwnerBasisAffinityDenial {
    TargetBasis,
    Owner {
        descriptor_graph_instance_id: String,
        runtime_graph_instance_id: String,
    },
    Definition {
        descriptor_definition_basis: u64,
        runtime_definition_basis: u64,
    },
}

fn map_managed_basis_affinity_denial(
    denial: SignalOwnerBasisAffinityDenial,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        SignalOwnerBasisAffinityDenial::TargetBasis => {
            SignalBranchBasisReadmissionDenial::ReferenceMismatch {
                axes: vec![FoundationalBranchReferenceMismatchAxis::TargetBasis],
            }
        }
        SignalOwnerBasisAffinityDenial::Owner {
            descriptor_graph_instance_id,
            runtime_graph_instance_id,
        } => SignalBranchBasisReadmissionDenial::OwnerMismatch {
            descriptor_graph_instance_id,
            runtime_graph_instance_id,
        },
        SignalOwnerBasisAffinityDenial::Definition {
            descriptor_definition_basis,
            runtime_definition_basis,
        } => SignalBranchBasisReadmissionDenial::DefinitionMismatch {
            descriptor_definition_basis,
            runtime_definition_basis,
        },
    }
}

fn map_retained_basis_affinity_denial(
    denial: SignalOwnerBasisAffinityDenial,
) -> SignalBranchRetainedReadmissionDenial {
    SignalBranchRetainedReadmissionDenial::UnavailableExactTarget(
        map_retention_basis_affinity_denial(denial),
    )
}

fn map_retention_basis_affinity_denial(
    denial: SignalOwnerBasisAffinityDenial,
) -> SignalBranchRetentionAcquisitionDenial {
    match denial {
        SignalOwnerBasisAffinityDenial::TargetBasis
        | SignalOwnerBasisAffinityDenial::Owner { .. } => {
            SignalBranchRetentionAcquisitionDenial::ForeignBasis
        }
        SignalOwnerBasisAffinityDenial::Definition {
            descriptor_definition_basis,
            runtime_definition_basis,
        } => SignalBranchRetentionAcquisitionDenial::DefinitionMismatch {
            basis_definition_basis: descriptor_definition_basis,
            runtime_definition_basis,
        },
    }
}
