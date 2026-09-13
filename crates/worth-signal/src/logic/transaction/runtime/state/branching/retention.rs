use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisDescriptor, SignalBranchBasisLifecyclePosture,
    SignalBranchRetainedReadmissionDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionLease, SignalBranchRetentionOwnerRelationship,
    SignalBranchRetentionReleaseDenial, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetentionTerminalCounts, SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
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
    /// Open one explicit external obligation over the exact immutable target
    /// the admitted basis names.
    ///
    /// This deliberately does not compare the basis to the branch's current
    /// observation. An exact obligation is a statement about one immutable
    /// target, so it stays legitimate after the branch advances; the branch's
    /// currentness is the concern of mutation and ordinary readmission, not of
    /// component retention.
    pub fn retain_signal_component_basis(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchRetentionLease, SignalBranchRetentionAcquisitionDenial> {
        if let Some((basis_port, _, _)) = self.sealed_owner_port_slots() {
            return basis_port.retain_exact(basis);
        }
        let descriptor = basis.descriptor();
        self.validate_exact_retention_target(descriptor)?;
        self.branches.acquire_retention(basis)
    }

    /// Consume one external obligation issued by this runtime.
    ///
    /// A foreign obligation is refused with the still-live lease handed back,
    /// so a caller that addressed the wrong owner keeps its retention.
    pub fn release_signal_component_basis(
        &self,
        lease: SignalBranchRetentionLease,
    ) -> SignalBranchRetentionReleaseOutcome {
        if let Some((basis_port, _, _)) = self.sealed_owner_port_slots() {
            return basis_port.release_exact(lease);
        }
        let binding = self.branches.retention_binding();
        match lease.owner_relationship(&binding) {
            SignalBranchRetentionOwnerRelationship::DifferentOwner => {
                SignalBranchRetentionReleaseOutcome::Denied {
                    lease,
                    denial: SignalBranchRetentionReleaseDenial::ForeignRuntime,
                }
            }
            SignalBranchRetentionOwnerRelationship::SameOwner
            | SignalBranchRetentionOwnerRelationship::OwnerLost => {
                SignalBranchRetentionReleaseOutcome::Released(lease.release())
            }
        }
    }

    /// Readmit the exact basis one live external obligation retains.
    ///
    /// Unlike [`SignalRuntime::readmit_signal_branch_basis`], this admits the
    /// retained historical target rather than the branch's current one. The
    /// obligation is the authority: it proves the owner still holds that exact
    /// target available.
    pub fn readmit_retained_signal_branch_basis(
        &self,
        descriptor: SignalBranchBasisDescriptor,
        lease: &SignalBranchRetentionLease,
    ) -> Result<AdmittedSignalBranchBasis, SignalBranchRetainedReadmissionDenial> {
        if let Some((basis_port, _, _)) = self.sealed_owner_port_slots() {
            return basis_port.readmit_retained_exact(&descriptor, lease);
        }
        match lease.owner_relationship(&self.branches.retention_binding()) {
            SignalBranchRetentionOwnerRelationship::DifferentOwner => {
                return Err(SignalBranchRetainedReadmissionDenial::ForeignRetention)
            }
            SignalBranchRetentionOwnerRelationship::OwnerLost => {
                return Err(SignalBranchRetainedReadmissionDenial::UnavailableRetainedTarget)
            }
            SignalBranchRetentionOwnerRelationship::SameOwner => {}
        }
        if !lease.retains_live_obligation() {
            return Err(SignalBranchRetainedReadmissionDenial::UnavailableRetainedTarget);
        }
        if &descriptor != lease.descriptor() {
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
        self.validate_exact_retention_target(&descriptor)
            .map_err(SignalBranchRetainedReadmissionDenial::UnavailableExactTarget)?;
        let branch_id = descriptor.branch_id();
        self.admit_unsealed_canonical_basis_with_retention(
            descriptor.observation().clone(),
            branch_id,
            || self.branches.acquire_admitted_retention(branch_id),
        )
        .map_err(|denial| match denial {
            SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(unavailable) => {
                SignalBranchRetainedReadmissionDenial::OwnerUnavailable(unavailable)
            }
            SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            } => SignalBranchRetainedReadmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            },
            SignalBranchRetentionAcquisitionDenial::OwnerReentry => {
                SignalBranchRetainedReadmissionDenial::OwnerReentry
            }
            SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases,
            } => SignalBranchRetainedReadmissionDenial::UnavailableRetention {
                maximum_active_leases,
            },
            SignalBranchRetentionAcquisitionDenial::IdentityExhausted => {
                SignalBranchRetainedReadmissionDenial::RetentionIdentityExhausted
            }
            denial => SignalBranchRetainedReadmissionDenial::UnavailableExactTarget(denial),
        })
    }

    /// Terminality this runtime's narrow retention owner has recorded for
    /// external component obligations.
    pub fn signal_component_retention_terminal_counts(
        &self,
    ) -> SignalBranchRetentionTerminalCounts {
        if self.owner_services.is_sealed() {
            return self.owner_services.legacy_retention_terminal_counts();
        }
        self.branches.retention_terminal_counts()
    }

    /// Decide every exact-target obligation this owner can decide: runtime
    /// affinity, branch lifecycle, definition agreement, and continued
    /// availability of the exact immutable target. Currentness is not among
    /// them.
    fn validate_exact_retention_target(
        &self,
        descriptor: &SignalBranchBasisDescriptor,
    ) -> Result<(), SignalBranchRetentionAcquisitionDenial> {
        let Some(target) = descriptor.observation().target().as_basis() else {
            return Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis);
        };
        if target.graph_instance_id() != self.branches.owner_runtime_instance_id().to_string() {
            return Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis);
        }
        let runtime_definition_basis = signal_definition_basis(self);
        if target.definition_basis() != runtime_definition_basis {
            return Err(SignalBranchRetentionAcquisitionDenial::DefinitionMismatch {
                basis_definition_basis: target.definition_basis(),
                runtime_definition_basis,
            });
        }
        let branch_id = descriptor.branch_id();
        if self.branches.branch_handle(branch_id).is_none() {
            return if self.branches.branch_retirement_receipt(branch_id).is_some() {
                Err(SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id })
            } else {
                Err(SignalBranchRetentionAcquisitionDenial::UnknownBranch { branch_id })
            };
        }
        if let Some(snapshot_id) = target.snapshot_id() {
            let snapshot_id = SignalSnapshotId(snapshot_id);
            if self
                .branches
                .snapshot_state(branch_id, snapshot_id)
                .is_none()
                && self.branches.branch_head_snapshot_id(branch_id) != Some(snapshot_id)
            {
                return Err(SignalBranchRetentionAcquisitionDenial::UnavailableTarget {
                    branch_id,
                    snapshot_id,
                });
            }
        }
        Ok(())
    }
}
