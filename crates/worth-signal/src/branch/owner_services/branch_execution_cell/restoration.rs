use std::sync::atomic::Ordering;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchObservation,
    SignalBranchRestoreDenial,
};
use crate::data::error::SignalError;

use super::super::owner_metadata::SignalOwnerSnapshotStateBinding;
use super::super::{
    SignalBranchCellState, SignalOwnerCancellationToken, SignalOwnerOperationAdmission,
    SignalOwnerUnavailable,
};
use super::{SignalBranchCellAdmissionDenial, SignalBranchCellWork, SignalBranchExecutionCell};
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SignalRestoreStateTruth {
    pub(crate) handle: crate::state::SignalBranchHandle,
    pub(crate) graph: serde_json::Value,
    pub(crate) mutation_ledger: crate::logic::transaction::BranchMutationLedger,
    pub(crate) generation: u64,
    pub(crate) restore_snapshot_id: Option<crate::state::SignalSnapshotId>,
    pub(crate) observation: SignalBranchObservation,
    pub(crate) dependency_sources: Vec<crate::data::handle::NodeId>,
}

pub(crate) struct SignalBranchRestoreCellOutcome {
    branch_id: crate::state::SignalBranchId,
    observation: SignalBranchObservation,
}

impl SignalBranchRestoreCellOutcome {
    pub(crate) fn into_observation(self) -> SignalBranchObservation {
        self.observation
    }

    pub(in crate::branch::owner_services) fn into_output_parts(
        self,
    ) -> (crate::state::SignalBranchId, SignalBranchObservation) {
        (self.branch_id, self.observation)
    }
}

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[cfg(test)]
    pub(crate) fn restore_state_truth_after_fault(
        &self,
        dependent: crate::data::handle::NodeId,
    ) -> SignalRestoreStateTruth {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        SignalRestoreStateTruth {
            handle: state.handle().clone(),
            graph: serde_json::to_value(state.state().graph())
                .expect("restored graph serializes for exact fault observation"),
            mutation_ledger: state.state().mutation_ledger().clone(),
            generation: state.head_generation(),
            restore_snapshot_id: state.restore_snapshot_id(),
            observation: state
                .observation()
                .expect("restored state remains internally complete"),
            dependency_sources: state
                .state()
                .graph()
                .dependency_sources_of(dependent)
                .expect("restored dependency owner remains live"),
        }
    }

    /// Restores one owner-admitted snapshot into its exact target cell.
    pub(crate) fn restore_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        expected: &AdmittedSignalBranchBasis,
        admitted_snapshot: &AdmittedSignalBranchSnapshot,
        snapshot_state: SignalOwnerSnapshotStateBinding<D, I, T>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchRestoreCellOutcome, SignalBranchRestoreDenial> {
        self.restore_exact_with_observers(
            admission,
            expected,
            admitted_snapshot,
            snapshot_state,
            cancellation,
            || {},
            || {},
        )
    }

    pub(in crate::branch::owner_services) fn restore_exact_with_observers(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        expected: &AdmittedSignalBranchBasis,
        admitted_snapshot: &AdmittedSignalBranchSnapshot,
        snapshot_state: SignalOwnerSnapshotStateBinding<D, I, T>,
        cancellation: &SignalOwnerCancellationToken,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
    ) -> Result<SignalBranchRestoreCellOutcome, SignalBranchRestoreDenial> {
        if admitted_snapshot.owner_runtime_instance_id() != self.owner_runtime_instance_id {
            return Err(SignalBranchRestoreDenial::ForeignSnapshotOwner {
                expected_runtime_instance_id: self.owner_runtime_instance_id,
                observed_runtime_instance_id: admitted_snapshot.owner_runtime_instance_id(),
            });
        }
        let snapshot = admitted_snapshot.snapshot();
        if snapshot.meta.branch_id != self.branch_id {
            return Err(SignalBranchRestoreDenial::CrossBranchSnapshot {
                branch_id: self.branch_id,
                snapshot_branch_id: snapshot.meta.branch_id,
            });
        }
        let snapshot_state = snapshot_state.into_state_for(
            self.owner_runtime_instance_id,
            self.owner_lifecycle_identity,
            admitted_snapshot,
        )?;
        cancellation
            .preflight_cell_wait()
            .map_err(|_| SignalBranchRestoreDenial::CancelledNoMovement)?;
        self.validate_admission(admission)
            .map_err(|denial| map_restore_cell_denial(denial, self.branch_id))?;
        let _cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(|denial| map_restore_cell_denial(denial, self.branch_id))?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let mut state = self
            .lock_state_after_contention_observation()
            .map_err(|denial| map_restore_cell_denial(denial, self.branch_id))?;
        self.require_live_posture()
            .map_err(|denial| map_restore_cell_denial(denial, self.branch_id))?;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::ExactBasisPreflight);
        let live = state.observation().map_err(owner_restore_failure)?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchRestoreDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        let prepared = state
            .prepare_restore(snapshot_state, snapshot)
            .map_err(owner_restore_failure)?;
        let observation = prepared.observation.clone();
        before_movement();
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
        let permit = cancellation
            .preflight_movement()
            .map_err(|_| SignalBranchRestoreDenial::CancelledNoMovement)?;
        state.commit_restore(prepared);
        SignalBranchCellWork {
            counters: &self.counters,
            movements: &self.movements,
        }
        .record_canonical_movement(&permit);
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::AfterCanonicalMovement);
        after_movement();
        Ok(SignalBranchRestoreCellOutcome {
            branch_id: self.branch_id,
            observation,
        })
    }
}

pub(in crate::branch::owner_services) fn map_restore_cell_denial(
    denial: SignalBranchCellAdmissionDenial,
    branch_id: crate::state::SignalBranchId,
) -> SignalBranchRestoreDenial {
    match denial {
        SignalBranchCellAdmissionDenial::ForeignOwner
        | SignalBranchCellAdmissionDenial::ExpiredLifecycle => {
            SignalBranchRestoreDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchCellAdmissionDenial::SecondCellWhileHeld => {
            SignalBranchRestoreDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchCellAdmissionDenial::ExecutingThreadReentry => {
            SignalBranchRestoreDenial::OwnerReentry
        }
        SignalBranchCellAdmissionDenial::RetirementInProgress => {
            SignalBranchRestoreDenial::RetirementInProgress { branch_id }
        }
        SignalBranchCellAdmissionDenial::RetiredIncarnation => {
            SignalBranchRestoreDenial::RetiredBranch { branch_id }
        }
        SignalBranchCellAdmissionDenial::PoisonedIncarnation => {
            SignalBranchRestoreDenial::QuarantinedBranch { branch_id }
        }
    }
}

fn owner_restore_failure(error: SignalError) -> SignalBranchRestoreDenial {
    SignalBranchRestoreDenial::OwnerDeniedNoMovement { error }
}
