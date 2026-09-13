use std::sync::atomic::Ordering;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchObservation, SignalBranchSnapshotCaptureDenial,
};
use crate::data::error::SignalError;
use crate::state::SignalSnapshotV1;

use super::super::owner_metadata::SignalOwnerSnapshotReservation;
use super::super::{SignalBranchCellState, SignalOwnerCancellationToken, SignalOwnerUnavailable};
use super::{SignalBranchCellAdmissionDenial, SignalBranchCellWork, SignalBranchExecutionCell};
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

pub(crate) struct SignalBranchSnapshotCellOutcome {
    branch_id: crate::state::SignalBranchId,
    snapshot: SignalSnapshotV1,
    observation: SignalBranchObservation,
}

impl SignalBranchSnapshotCellOutcome {
    pub(crate) fn snapshot(&self) -> &SignalSnapshotV1 {
        &self.snapshot
    }

    pub(crate) fn observation(&self) -> &SignalBranchObservation {
        &self.observation
    }

    pub(crate) fn into_parts(self) -> (SignalSnapshotV1, SignalBranchObservation) {
        (self.snapshot, self.observation)
    }

    pub(in crate::branch::owner_services) fn into_output_parts(
        self,
    ) -> (
        crate::state::SignalBranchId,
        SignalSnapshotV1,
        SignalBranchObservation,
    ) {
        (self.branch_id, self.snapshot, self.observation)
    }
}

impl<D, I, T> SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Captures and installs one snapshot through a capacity reservation made
    /// before cell admission. Metadata installation occurs after cell release.
    pub(crate) fn capture_snapshot_exact(
        &self,
        expected: &AdmittedSignalBranchBasis,
        reservation: SignalOwnerSnapshotReservation<'_, D, I, T>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchSnapshotCellOutcome, SignalBranchSnapshotCaptureDenial> {
        cancellation
            .preflight_cell_wait()
            .map_err(|_| SignalBranchSnapshotCaptureDenial::CancelledNoMovement)?;
        let admission = reservation.admission();
        self.validate_admission(admission)
            .map_err(|denial| map_snapshot_cell_denial(denial, self.branch_id))?;
        if !reservation.matches_cell(self) {
            return Err(SignalBranchSnapshotCaptureDenial::OwnerCellMisuse {
                branch_id: self.branch_id,
            });
        }
        let cell_hold = admission
            .hold_branch_cell()
            .map_err(SignalBranchCellAdmissionDenial::from)
            .map_err(|denial| map_snapshot_cell_denial(denial, self.branch_id))?;
        self.counters.record_target_cell_contact();
        self.contacts.fetch_add(1, Ordering::SeqCst);
        let mut state = self
            .lock_state_after_contention_observation()
            .map_err(|denial| map_snapshot_cell_denial(denial, self.branch_id))?;
        self.require_live_posture()
            .map_err(|denial| map_snapshot_cell_denial(denial, self.branch_id))?;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::ExactBasisPreflight);
        let live = state.observation().map_err(owner_snapshot_failure)?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchSnapshotCaptureDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        let mut prepared = state
            .prepare_snapshot(reservation.snapshot_id())
            .map_err(owner_snapshot_failure)?;
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
        let permit = cancellation
            .preflight_movement()
            .map_err(|_| SignalBranchSnapshotCaptureDenial::CancelledNoMovement)?;
        state.commit_snapshot(&mut prepared);
        SignalBranchCellWork {
            counters: &self.counters,
            movements: &self.movements,
        }
        .record_canonical_movement(&permit);
        drop(state);
        drop(cell_hold);
        reservation.install(prepared.packet);
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::AfterCanonicalMovement);
        Ok(SignalBranchSnapshotCellOutcome {
            branch_id: self.branch_id,
            snapshot: prepared.snapshot,
            observation: prepared.observation,
        })
    }
}

pub(in crate::branch::owner_services) fn map_snapshot_cell_denial(
    denial: SignalBranchCellAdmissionDenial,
    branch_id: crate::state::SignalBranchId,
) -> SignalBranchSnapshotCaptureDenial {
    match denial {
        SignalBranchCellAdmissionDenial::ForeignOwner
        | SignalBranchCellAdmissionDenial::ExpiredLifecycle => {
            SignalBranchSnapshotCaptureDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchCellAdmissionDenial::SecondCellWhileHeld => {
            SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchCellAdmissionDenial::ExecutingThreadReentry => {
            SignalBranchSnapshotCaptureDenial::OwnerReentry
        }
        SignalBranchCellAdmissionDenial::RetirementInProgress => {
            SignalBranchSnapshotCaptureDenial::RetirementInProgress { branch_id }
        }
        SignalBranchCellAdmissionDenial::RetiredIncarnation => {
            SignalBranchSnapshotCaptureDenial::RetiredBranch { branch_id }
        }
        SignalBranchCellAdmissionDenial::PoisonedIncarnation => {
            SignalBranchSnapshotCaptureDenial::QuarantinedBranch { branch_id }
        }
    }
}

fn owner_snapshot_failure(error: SignalError) -> SignalBranchSnapshotCaptureDenial {
    SignalBranchSnapshotCaptureDenial::OwnerDeniedNoMovement { error }
}
