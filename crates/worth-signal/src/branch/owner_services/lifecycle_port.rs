use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, PlannedSignalBranchRetirement,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
};

use super::{
    SignalOwner, SignalOwnerAdmissionDenial, SignalOwnerCancellationToken,
    SignalOwnerLifecycleObservation, SignalOwnerServiceCostSnapshot, SignalOwnerUnavailable,
};

#[cfg(test)]
mod tests;

/// Public concrete weak lifecycle service issued by a sealed owner root.
pub struct SignalBranchLifecyclePort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Weak<SignalOwner<D, I, T>>,
    diagnostic_owner_runtime_instance_id: u64,
}

impl<D, I, T> Clone for SignalBranchLifecyclePort<D, I, T>
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

impl<D, I, T> SignalBranchLifecyclePort<D, I, T>
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

    pub(super) fn upgrade_owner(
        &self,
    ) -> Result<Arc<SignalOwner<D, I, T>>, SignalOwnerUnavailable> {
        SignalOwner::upgrade(&self.owner)
    }

    pub fn plan_retirement_exact(
        &self,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let owner = match self.upgrade_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::OwnerUnavailable(
                    unavailable,
                ))
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => return TransitionOutcome::denied(map_admission_denial(denial)),
        };
        owner.plan_retirement_exact(&admission, expected, reason)
    }

    pub fn plan_retirement_releasing_snapshots_exact(
        &self,
        expected: AdmittedSignalBranchBasis,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let owner = match self.upgrade_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::OwnerUnavailable(
                    unavailable,
                ))
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => return TransitionOutcome::denied(map_admission_denial(denial)),
        };
        owner.plan_retirement_releasing_snapshots_exact(
            &admission,
            expected,
            releasing_snapshots,
            reason,
        )
    }

    pub fn retire_exact(
        &self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
    ) -> TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        let owner = match self.upgrade_owner() {
            Ok(owner) => owner,
            Err(unavailable) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::OwnerUnavailable(
                    unavailable,
                ))
            }
        };
        let admission = match owner.admit() {
            Ok(admission) => admission,
            Err(denial) => return TransitionOutcome::denied(map_admission_denial(denial)),
        };
        if !owner.basis_has_owner_affinity(plan.admitted_basis()) {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let branch_id = plan.branch().id;
        let reservation = match owner.reserve_retirement(&admission, branch_id) {
            Ok(reservation) => reservation,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        match reservation.execute(plan, cancellation) {
            Ok(receipt) => TransitionOutcome::success(receipt),
            Err(denial) => TransitionOutcome::denied(denial),
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
}

pub(in crate::branch::owner_services) fn map_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchRetirementDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchRetirementDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => SignalBranchRetirementDenial::OwnerReentry,
    }
}
