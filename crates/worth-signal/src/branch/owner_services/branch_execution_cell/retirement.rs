use std::sync::atomic::Ordering;

use crate::branch::{
    PlannedSignalBranchRetirement, SignalBranchRetirementDenial, SignalBranchRetirementReceipt,
};
use crate::logic::transaction::canonical_digest;

use super::super::{
    SignalBranchCellState, SignalOwnerCancellationToken, SignalOwnerOperationAdmission,
    SignalOwnerUnavailable,
};
use super::{
    SignalBranchCellAdmissionDenial, SignalBranchCellRetirementPosture, SignalBranchCellWork,
    SignalBranchExecutionCell, CELL_LIVE, CELL_RETIRED, CELL_RETIRING,
};
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

pub(crate) struct SignalBranchRetirementCellOutcome {
    pub(crate) receipt: SignalBranchRetirementReceipt,
}

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Consumes one owner-issued plan under its sole target-cell incarnation.
    pub(crate) fn retire_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchRetirementCellOutcome, SignalBranchRetirementDenial> {
        self.retire_exact_with_receipt_capacity(admission, plan, cancellation, 0, || {}, || {})
    }

    pub(in crate::branch::owner_services) fn retire_exact_with_receipt_capacity(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
        reclaimed_snapshot_state_count: u32,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
    ) -> Result<SignalBranchRetirementCellOutcome, SignalBranchRetirementDenial> {
        cancellation
            .preflight_cell_wait()
            .map_err(|_| SignalBranchRetirementDenial::CancelledNoMovement)?;
        self.validate_admission(admission)
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let state = self
            .lock_state_after_contention_observation()
            .map_err(|denial| map_retirement_cell_denial(denial, self.branch_id))?;
        self.lifecycle_posture
            .compare_exchange(
                CELL_LIVE,
                CELL_RETIRING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map_err(|observed| match observed {
                CELL_RETIRING => SignalBranchRetirementDenial::RetirementInProgress {
                    branch_id: self.branch_id,
                },
                CELL_RETIRED => SignalBranchRetirementDenial::RetiredBranch {
                    branch_id: self.branch_id,
                },
                _ => unreachable!("branch cell lifecycle posture is owner-defined"),
            })?;
        let mut posture = SignalBranchCellRetirementPosture {
            lifecycle_posture: &self.lifecycle_posture,
            retired: false,
        };
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::ExactBasisPreflight);
        if plan.branch().id != state.branch_id()
            || state.observation().map_or(true, |live| {
                live.compare(plan.admitted_basis().observation()).is_err()
            })
        {
            return Err(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        before_movement();
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
        let permit = cancellation
            .preflight_movement()
            .map_err(|_| SignalBranchRetirementDenial::CancelledNoMovement)?;
        let retired_branch = state.handle().clone();
        let Some(parent_branch_id) = retired_branch.parent_branch_id else {
            return Err(SignalBranchRetirementDenial::CanonicalBranch {
                branch_id: retired_branch.id,
            });
        };
        let forked_from_snapshot_id = state.state().owner_retirement_forked_from_snapshot_id();
        let terminal_head_snapshot_id = retired_branch.head_snapshot_id;
        let reason = plan.reason();
        let terminal_basis_digest = plan.terminal_basis_digest.clone();
        let closeout_digest = canonical_digest(&(
            retired_branch.id,
            parent_branch_id,
            forked_from_snapshot_id,
            terminal_head_snapshot_id,
            reason,
            terminal_basis_digest.as_str(),
        ));
        let receipt = SignalBranchRetirementReceipt {
            retired_branch: retired_branch.clone(),
            parent_branch_id,
            forked_from_snapshot_id,
            terminal_head_snapshot_id,
            reason,
            terminal_basis_digest: terminal_basis_digest.clone(),
            closeout_digest,
            reclaimed_branch_state_count: 1,
            reclaimed_snapshot_state_count,
            reclaimed_runtime_meta_count: 0,
            retained_proof_record_count: 1,
        };
        SignalBranchCellWork {
            counters: &self.counters,
            movements: &self.movements,
        }
        .record_canonical_movement(&permit);
        *self
            .retirement_receipt
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(receipt.clone());
        self.lifecycle_posture
            .store(CELL_RETIRED, Ordering::Release);
        posture.retired = true;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::AfterCanonicalMovement);
        after_movement();
        Ok(SignalBranchRetirementCellOutcome { receipt })
    }
}

pub(in crate::branch::owner_services) fn map_retirement_cell_denial(
    denial: SignalBranchCellAdmissionDenial,
    branch_id: crate::state::SignalBranchId,
) -> SignalBranchRetirementDenial {
    match denial {
        SignalBranchCellAdmissionDenial::ForeignOwner
        | SignalBranchCellAdmissionDenial::ExpiredLifecycle => {
            SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchCellAdmissionDenial::SecondCellWhileHeld => {
            SignalBranchRetirementDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchCellAdmissionDenial::ExecutingThreadReentry => {
            SignalBranchRetirementDenial::OwnerReentry
        }
        SignalBranchCellAdmissionDenial::RetirementInProgress => {
            SignalBranchRetirementDenial::RetirementInProgress { branch_id }
        }
        SignalBranchCellAdmissionDenial::RetiredIncarnation => {
            SignalBranchRetirementDenial::RetiredBranch { branch_id }
        }
        SignalBranchCellAdmissionDenial::PoisonedIncarnation => {
            SignalBranchRetirementDenial::QuarantinedBranch { branch_id }
        }
    }
}
