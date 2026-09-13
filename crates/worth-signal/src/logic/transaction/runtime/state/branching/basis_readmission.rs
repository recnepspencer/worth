use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisCompatibilityDenial, SignalBranchBasisDescriptor,
    SignalBranchBasisLifecyclePosture, SignalBranchBasisReadmissionDenial,
    SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
};
use crate::state::SignalSnapshotId;

use super::super::runtime_state::SignalRuntime;
use super::basis_definition::signal_definition_basis;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn readmit_signal_branch_basis(
        &self,
        descriptor: SignalBranchBasisDescriptor,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchBasisReadmissionDenial> {
        if self.owner_services.is_sealed() {
            return self.owner_services.readmit_legacy_descriptor(descriptor);
        }
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
        let branch_id = descriptor.owner_branch_id();
        let Some(target) = descriptor.observation().target().as_basis() else {
            return Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch {
                axes: vec![
                    worth_foundational::FoundationalBranchReferenceMismatchAxis::TargetBasis,
                ],
            });
        };
        let runtime_graph_instance_id = self.branches.owner_runtime_instance_id().to_string();
        if target.graph_instance_id() != runtime_graph_instance_id {
            return Err(SignalBranchBasisReadmissionDenial::OwnerMismatch {
                descriptor_graph_instance_id: target.graph_instance_id().to_owned(),
                runtime_graph_instance_id,
            });
        }
        let runtime_definition_basis = signal_definition_basis(self);
        if target.definition_basis() != runtime_definition_basis {
            return Err(SignalBranchBasisReadmissionDenial::DefinitionMismatch {
                descriptor_definition_basis: target.definition_basis(),
                runtime_definition_basis,
            });
        }
        let Some(branch) = self.branches.branch_handle(branch_id) else {
            return if self.branches.branch_retirement_receipt(branch_id).is_some() {
                Err(SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id })
            } else {
                Err(SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id })
            };
        };
        if let Some(snapshot_id) = target.snapshot_id() {
            let snapshot_id = SignalSnapshotId(snapshot_id);
            if self
                .branches
                .snapshot_state(branch_id, snapshot_id)
                .is_none()
                && self.branches.branch_head_snapshot_id(branch_id) != Some(snapshot_id)
            {
                return Err(SignalBranchBasisReadmissionDenial::UnavailableSnapshot {
                    branch_id,
                    snapshot_id,
                });
            }
        }
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = descriptor.observation().compare(&live) {
            return Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        self.admit_unsealed_canonical_basis_with_retention(
            descriptor.observation().clone(),
            branch_id,
            || self.branches.acquire_admitted_retention(branch_id),
        )
        .map_err(|denial| match denial {
            crate::branch::SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(
                unavailable,
            ) => SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable),
            crate::branch::SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            } => SignalBranchBasisReadmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            },
            crate::branch::SignalBranchRetentionAcquisitionDenial::OwnerReentry => {
                SignalBranchBasisReadmissionDenial::OwnerReentry
            }
            crate::branch::SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases,
            } => SignalBranchBasisReadmissionDenial::UnavailableRetention {
                maximum_active_leases,
            },
            crate::branch::SignalBranchRetentionAcquisitionDenial::IdentityExhausted => {
                SignalBranchBasisReadmissionDenial::RetentionIdentityExhausted
            }
            crate::branch::SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id } => {
                SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id }
            }
            crate::branch::SignalBranchRetentionAcquisitionDenial::UnknownBranch { branch_id } => {
                SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id }
            }
            _ => SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id },
        })
    }

    pub fn validate_signal_basis_compatibility(
        &self,
        left: &AdmittedSignalBranchBasis,
        right: &AdmittedSignalBranchBasis,
    ) -> Result<(), SignalBranchBasisCompatibilityDenial> {
        let Some(left_target) = left.observation().target().as_basis() else {
            return Err(SignalBranchBasisCompatibilityDenial::OwnerMismatch);
        };
        let Some(right_target) = right.observation().target().as_basis() else {
            return Err(SignalBranchBasisCompatibilityDenial::OwnerMismatch);
        };
        if left_target.graph_instance_id() != right_target.graph_instance_id() {
            return Err(SignalBranchBasisCompatibilityDenial::OwnerMismatch);
        }
        if left_target.definition_basis() != right_target.definition_basis() {
            return Err(SignalBranchBasisCompatibilityDenial::DefinitionMismatch);
        }
        if left_target.snapshot_id() != right_target.snapshot_id() {
            return Err(SignalBranchBasisCompatibilityDenial::SnapshotMismatch);
        }
        if left_target.restore_snapshot_id() != right_target.restore_snapshot_id() {
            return Err(SignalBranchBasisCompatibilityDenial::RestoreMismatch);
        }
        Ok(())
    }
}
