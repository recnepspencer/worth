use std::sync::{Arc, TryLockError};

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::SignalBranchBasisObservationDenial;
use crate::data::error::SignalError;
use crate::data::retained_storage::SignalConditionalRetentionDenial;

use super::super::{
    SignalConditionalEvaluationAdmission, SignalConditionalEvaluationState,
    SignalConditionalExecutionPort, SignalConditionalExecutionSlot,
};
use super::SignalConditionalSuccessorTransition;

pub struct SignalConditionalEvaluationReadmissionRequest<'a> {
    pub predecessor: &'a SignalConditionalEvaluationAdmission,
    pub transitions: &'a [&'a SignalConditionalSuccessorTransition],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignalConditionalEvaluationReadmissionCounters {
    transitions_checked: usize,
    targets_applied: usize,
    persistent_roots_forked: usize,
    source_opens: usize,
    unrelated_lowering_scans: usize,
}

impl SignalConditionalEvaluationReadmissionCounters {
    pub(in crate::branch::owner_services) const fn new(
        transitions_checked: usize,
        targets_applied: usize,
        persistent_roots_forked: usize,
    ) -> Self {
        Self {
            transitions_checked,
            targets_applied,
            persistent_roots_forked,
            source_opens: 0,
            unrelated_lowering_scans: 0,
        }
    }

    pub const fn transitions_checked(self) -> usize {
        self.transitions_checked
    }

    pub const fn targets_applied(self) -> usize {
        self.targets_applied
    }

    pub const fn persistent_roots_forked(self) -> usize {
        self.persistent_roots_forked
    }

    pub const fn source_opens(self) -> usize {
        self.source_opens
    }

    pub const fn unrelated_lowering_scans(self) -> usize {
        self.unrelated_lowering_scans
    }
}

#[derive(Debug)]
pub enum SignalConditionalEvaluationReadmissionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    StaleBasisAdmission,
    DefinitionReadmissionRequired,
    DefinitionMismatch,
    TransitionChainIncomplete,
    TransitionChainMismatch,
    PredecessorNotExecuted,
    EvaluationIdentityExhausted,
    AdmissionCapacityExhausted,
    AdmissionUnavailable,
    SlotBusy,
    SlotPoisoned,
    SlotAdmission(SignalError),
    UnconsumedUnwind,
}

pub struct SignalConditionalEvaluationReadmission {
    admission: SignalConditionalEvaluationAdmission,
    counters: SignalConditionalEvaluationReadmissionCounters,
}

impl SignalConditionalEvaluationReadmission {
    pub fn into_parts(
        self,
    ) -> (
        SignalConditionalEvaluationAdmission,
        SignalConditionalEvaluationReadmissionCounters,
    ) {
        (self.admission, self.counters)
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn readmit_evaluation(
        &self,
        request: SignalConditionalEvaluationReadmissionRequest<'_>,
    ) -> Result<SignalConditionalEvaluationReadmission, SignalConditionalEvaluationReadmissionDenial>
    {
        use SignalConditionalEvaluationReadmissionDenial as Denial;

        if !Arc::ptr_eq(&self.authority, &request.predecessor.service_authority) {
            return Err(Denial::DefinitionMismatch);
        }
        let mut predecessor = match request.predecessor.execution.try_lock() {
            Ok(execution) => execution,
            Err(TryLockError::WouldBlock) => return Err(Denial::SlotBusy),
            Err(TryLockError::Poisoned(_)) => return Err(Denial::SlotPoisoned),
        };
        if predecessor.slot.is_none() {
            return Err(Denial::PredecessorNotExecuted);
        }
        let target_basis = request
            .transitions
            .last()
            .map(|transition| Arc::clone(&transition.inner.successor))
            .unwrap_or_else(|| Arc::clone(&request.predecessor.retained_basis));
        let ordinal = self
            .next_evaluation_ordinal
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .map_err(|_| Denial::EvaluationIdentityExhausted)?;
        let execution_identity: Arc<str> = Arc::from(format!("signal-evaluation:{ordinal}"));
        let charge = super::super::admission_retention::evaluation_admission_charge(
            &request.predecessor.contract,
            &request.predecessor.source,
            &execution_identity,
        )
        .map_err(|_| Denial::AdmissionCapacityExhausted)?;
        let mut admission_custody = target_basis
            .reserve_evaluation_admission(charge)
            .map_err(map_retention_denial)?;

        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        let prepared = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.readmit_conditional_evaluation(
                &admission,
                &self.basis,
                &self.definition,
                &self.authority,
                request.predecessor,
                &mut predecessor,
                request.transitions,
                &target_basis,
                &mut admission_custody,
            )
        }));
        let (contract_binding, partition, counters) = match prepared {
            Ok(result) => result?,
            Err(payload) => {
                drop(predecessor);
                std::panic::resume_unwind(payload)
            }
        };
        Ok(SignalConditionalEvaluationReadmission {
            admission: SignalConditionalEvaluationAdmission {
                service_authority: Arc::clone(&self.authority),
                contract_binding,
                contract: request.predecessor.contract.clone(),
                source: Arc::clone(&request.predecessor.source),
                execution_identity,
                retained_basis: target_basis,
                execution: std::sync::Mutex::new(SignalConditionalEvaluationState {
                    admission_custody,
                    slot: Some(SignalConditionalExecutionSlot { partition }),
                    has_completed_execution: predecessor.has_completed_execution,
                }),
            },
            counters,
        })
    }
}

fn map_retention_denial(
    denial: SignalConditionalRetentionDenial,
) -> SignalConditionalEvaluationReadmissionDenial {
    match denial {
        SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalConditionalEvaluationReadmissionDenial::AdmissionCapacityExhausted
        }
        SignalConditionalRetentionDenial::Closed => {
            SignalConditionalEvaluationReadmissionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalConditionalRetentionDenial::Poisoned
        | SignalConditionalRetentionDenial::InvalidTransfer => {
            SignalConditionalEvaluationReadmissionDenial::AdmissionUnavailable
        }
    }
}
