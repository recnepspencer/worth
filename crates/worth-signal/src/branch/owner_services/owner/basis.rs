use std::sync::Arc;

use crate::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference,
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchObservation,
};
use crate::state::SignalBranchId;

use super::SignalOwner;
use crate::branch::owner_services::{
    SignalBranchRegistryDenial, SignalOwnerOperationAdmission, SignalOwnerUnavailable,
};

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admit_canonical_basis(
        &self,
        observation: SignalBranchObservation,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        retention: crate::branch::SignalBranchAdmissionLease,
    ) -> AdmittedSignalBranchBasis {
        self.basis_registry.admit(
            self.runtime_instance_id,
            self.definition_basis,
            branch_id,
            cell_incarnation,
            observation,
            retention,
        )
    }

    pub(in crate::branch::owner_services) fn admit_canonical_basis_with_retention<Acquire>(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        observation: SignalBranchObservation,
        branch_id: SignalBranchId,
        cell_incarnation: u64,
        acquire_retention: Acquire,
    ) -> Result<AdmittedSignalBranchBasis, crate::branch::SignalBranchRetentionAcquisitionDenial>
    where
        Acquire: FnOnce() -> Result<
            crate::branch::SignalBranchAdmissionLease,
            crate::branch::SignalBranchRetentionAcquisitionDenial,
        >,
    {
        self.basis_registry.admit_with_retention(
            self.runtime_instance_id,
            self.definition_basis,
            branch_id,
            cell_incarnation,
            observation,
            |_| self.validate_canonical_basis_reuse(admission, branch_id),
            acquire_retention,
        )
    }

    #[allow(
        dead_code,
        reason = "Phase 4 basis port publishes this private owner seam"
    )]
    pub(in crate::branch::owner_services) fn issue_managed_branch_reference(
        self: &Arc<Self>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial> {
        let target = basis
            .observation()
            .target()
            .as_basis()
            .ok_or(ManagedSignalBranchReferenceAdmissionDenial::OwnerInvariantViolation)?;
        if target.graph_instance_id() != self.runtime_instance_id().to_string()
            || target.definition_basis() != self.definition_basis()
        {
            return Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner);
        }
        let admission = self.admit().map_err(|denial| {
            match denial {
            crate::branch::owner_services::SignalOwnerAdmissionDenial::ForeignOwner => {
                ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner
            }
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OwnerUnavailable => {
                ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(
                    SignalOwnerUnavailable,
                )
            }
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            } => ManagedSignalBranchReferenceAdmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            },
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OwnerReentry => {
                ManagedSignalBranchReferenceAdmissionDenial::OwnerReentry
            }
        }
        })?;
        self.issue_managed_branch_reference_with_admission(&admission, basis)
    }

    pub(in crate::branch::owner_services) fn issue_managed_branch_reference_with_admission(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial> {
        let target = basis
            .observation()
            .target()
            .as_basis()
            .ok_or(ManagedSignalBranchReferenceAdmissionDenial::OwnerInvariantViolation)?;
        if target.graph_instance_id() != self.runtime_instance_id().to_string()
            || target.definition_basis() != self.definition_basis()
        {
            return Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner);
        }
        let cell = self
            .lookup_cell(admission, basis.owner_branch_id())
            .map_err(map_managed_reference_registry_denial)?;
        Ok(ManagedSignalBranchReference::owner_issued(
            &self.lifecycle,
            basis.owner_branch_id(),
            cell.incarnation(),
        ))
    }

    #[allow(
        dead_code,
        reason = "Phase 4 owner-service methods reuse this private admission seam"
    )]
    pub(in crate::branch::owner_services) fn admit_managed_branch_reference(
        self: &Arc<Self>,
        reference: &ManagedSignalBranchReference,
    ) -> Result<
        (
            SignalOwnerOperationAdmission<'_>,
            Arc<
                crate::branch::owner_services::SignalBranchExecutionCell<
                    crate::branch::owner_services::SignalBranchCellState<D, I, T>,
                >,
            >,
        ),
        ManagedSignalBranchReferenceAdmissionDenial,
    > {
        if !reference.is_bound_to(&self.lifecycle) {
            return Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner);
        }
        let admission = self.admit().map_err(|denial| {
            match denial {
            crate::branch::owner_services::SignalOwnerAdmissionDenial::ForeignOwner => {
                ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner
            }
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OwnerUnavailable => {
                ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(
                    SignalOwnerUnavailable,
                )
            }
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            } => ManagedSignalBranchReferenceAdmissionDenial::OperationCapacityExhausted {
                maximum_in_flight_operations,
            },
            crate::branch::owner_services::SignalOwnerAdmissionDenial::OwnerReentry => {
                ManagedSignalBranchReferenceAdmissionDenial::OwnerReentry
            }
        }
        })?;
        let cell = self.admit_managed_branch_reference_with_admission(&admission, reference)?;
        Ok((admission, cell))
    }

    pub(in crate::branch::owner_services) fn admit_managed_branch_reference_with_admission(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        reference: &ManagedSignalBranchReference,
    ) -> Result<
        Arc<
            crate::branch::owner_services::SignalBranchExecutionCell<
                crate::branch::owner_services::SignalBranchCellState<D, I, T>,
            >,
        >,
        ManagedSignalBranchReferenceAdmissionDenial,
    > {
        if !reference.is_bound_to(&self.lifecycle) {
            return Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner);
        }
        let cell = self
            .lookup_cell(admission, reference.branch_id())
            .map_err(map_managed_reference_registry_denial)?;
        if cell.incarnation() != reference.cell_incarnation() {
            return Err(ManagedSignalBranchReferenceAdmissionDenial::BranchIncarnationReplaced);
        }
        Ok(cell)
    }

    #[allow(
        dead_code,
        reason = "Phase 4 basis observation publishes this checked-cell seam"
    )]
    pub(in crate::branch::owner_services) fn observe_managed_branch_reference(
        self: &Arc<Self>,
        reference: &ManagedSignalBranchReference,
    ) -> Result<SignalBranchObservation, SignalBranchBasisObservationDenial> {
        let (admission, cell) = self
            .admit_managed_branch_reference(reference)
            .map_err(map_managed_reference_observation_denial)?;
        cell.observe_exact(&admission)
    }

    #[allow(
        dead_code,
        reason = "Phase 4 exact readmission consumes this checked-cell sub-step"
    )]
    pub(in crate::branch::owner_services) fn observe_managed_reference_for_readmission(
        self: &Arc<Self>,
        reference: &ManagedSignalBranchReference,
    ) -> Result<SignalBranchObservation, SignalBranchBasisReadmissionDenial> {
        let branch_id = reference.branch_id();
        let (admission, cell) = self
            .admit_managed_branch_reference(reference)
            .map_err(map_managed_reference_readmission_denial)?;
        cell.observe_exact(&admission)
            .map_err(|denial| map_observation_readmission_denial(denial, branch_id))
    }

    #[allow(dead_code, reason = "Phase 4 basis operations consume this owner seam")]
    pub(in crate::branch::owner_services) fn observe_branch_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalBranchObservation, SignalBranchBasisObservationDenial> {
        self.registry
            .lookup(admission, branch_id)
            .map_err(|denial| map_basis_registry_denial(denial, branch_id))?
            .observe_exact(admission)
    }
}

fn map_managed_reference_observation_denial(
    denial: ManagedSignalBranchReferenceAdmissionDenial,
) -> SignalBranchBasisObservationDenial {
    match denial {
        ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(unavailable)
        }
        denial => SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial },
    }
}

fn map_managed_reference_readmission_denial(
    denial: ManagedSignalBranchReferenceAdmissionDenial,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable)
        }
        denial => SignalBranchBasisReadmissionDenial::ManagedReferenceDenied { denial },
    }
}

fn map_observation_readmission_denial(
    denial: SignalBranchBasisObservationDenial,
    branch_id: SignalBranchId,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        SignalBranchBasisObservationDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable)
        }
        SignalBranchBasisObservationDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalBranchBasisObservationDenial::OwnerReentry => {
            SignalBranchBasisReadmissionDenial::OwnerReentry
        }
        SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial } => {
            SignalBranchBasisReadmissionDenial::ManagedReferenceDenied { denial }
        }
        SignalBranchBasisObservationDenial::UnknownBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::RetirementInProgress { branch_id } => {
            SignalBranchBasisReadmissionDenial::RetirementInProgress { branch_id }
        }
        SignalBranchBasisObservationDenial::RetiredBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::QuarantinedBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id } => {
            SignalBranchBasisReadmissionDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchBasisObservationDenial::OwnerInvariantViolation { branch_id } => {
            SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id }
        }
        SignalBranchBasisObservationDenial::InvalidOwnerObservation { .. } => {
            SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id }
        }
        SignalBranchBasisObservationDenial::RetentionUnavailable { denial } => match denial {
            crate::branch::SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases,
            } => SignalBranchBasisReadmissionDenial::UnavailableRetention {
                maximum_active_leases,
            },
            crate::branch::SignalBranchRetentionAcquisitionDenial::IdentityExhausted => {
                SignalBranchBasisReadmissionDenial::RetentionIdentityExhausted
            }
            _ => SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id },
        },
    }
}

fn map_managed_reference_registry_denial(
    denial: SignalBranchRegistryDenial,
) -> ManagedSignalBranchReferenceAdmissionDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            ManagedSignalBranchReferenceAdmissionDenial::BranchRetirementInProgress
        }
        SignalBranchRegistryDenial::TargetCellDenied(_) => {
            ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded
        }
        SignalBranchRegistryDenial::DuplicateBranch(_)
        | SignalBranchRegistryDenial::LiveCapacityExhausted { .. }
        | SignalBranchRegistryDenial::ReservationCapacityExhausted { .. }
        | SignalBranchRegistryDenial::NameAlreadyReserved
        | SignalBranchRegistryDenial::NameAlreadyInstalled
        | SignalBranchRegistryDenial::ExpiredRetirement(_) => {
            ManagedSignalBranchReferenceAdmissionDenial::OwnerInvariantViolation
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            ManagedSignalBranchReferenceAdmissionDenial::OwnerCellMisuse
        }
        SignalBranchRegistryDenial::OwnerReentry => {
            ManagedSignalBranchReferenceAdmissionDenial::OwnerReentry
        }
    }
}

pub(in crate::branch::owner_services) fn map_basis_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchBasisObservationDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchBasisObservationDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchBasisObservationDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            crate::branch::owner_services::branch_execution_cell::basis::map_basis_cell_denial(
                denial, branch_id,
            )
        }
        SignalBranchRegistryDenial::DuplicateBranch(_)
        | SignalBranchRegistryDenial::LiveCapacityExhausted { .. }
        | SignalBranchRegistryDenial::ReservationCapacityExhausted { .. }
        | SignalBranchRegistryDenial::NameAlreadyReserved
        | SignalBranchRegistryDenial::NameAlreadyInstalled
        | SignalBranchRegistryDenial::ExpiredRetirement(_) => {
            SignalBranchBasisObservationDenial::OwnerInvariantViolation { branch_id }
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => {
            SignalBranchBasisObservationDenial::OwnerReentry
        }
    }
}
