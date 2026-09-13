#[cfg(test)]
use super::SignalForkOutputReservation;
#[cfg(test)]
use crate::branch::{SignalBranchForkOperationDenial, ValidatedSignalBranchName};
#[cfg(test)]
use std::sync::Arc;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchRestoreDenial,
    SignalBranchSnapshotCaptureDenial, SignalBranchSnapshotCaptureOutcome,
};

use super::{SignalRestoreOutputReservation, SignalSnapshotOutputReservation};
use crate::branch::owner_services::branch_execution_cell::restoration::SignalBranchRestoreCellOutcome;
use crate::branch::owner_services::branch_execution_cell::snapshot::SignalBranchSnapshotCellOutcome;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
#[cfg(test)]
use crate::branch::owner_services::owner::fork_reservation::SignalInstalledOwnerFork;
use crate::branch::owner_services::SignalOwnerCancellationToken;
#[cfg(test)]
use crate::state::SignalBranchHandle;

pub(in crate::branch::owner_services) struct SignalReadySnapshotOutput<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    reservation: SignalSnapshotOutputReservation<'a, D, I, T>,
    outcome: SignalBranchSnapshotCellOutcome,
}

pub(in crate::branch::owner_services) struct SignalReadyRestoreOutput<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    reservation: SignalRestoreOutputReservation<'a, D, I, T>,
    outcome: SignalBranchRestoreCellOutcome,
}

#[cfg(test)]
pub(in crate::branch::owner_services) struct SignalReadyForkOutput<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    reservation: SignalForkOutputReservation<'a, D, I, T>,
    installed: SignalInstalledOwnerFork<'a, D, I, T>,
}

impl<'a, D, I, T> SignalSnapshotOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn capture(
        self,
        expected: &AdmittedSignalBranchBasis,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalReadySnapshotOutput<'a, D, I, T>, SignalBranchSnapshotCaptureDenial> {
        let snapshot = self
            .owner
            .metadata
            .reserve_snapshot(self.admission, &self.cell)?;
        let outcome = self
            .cell
            .capture_snapshot_exact(expected, snapshot, cancellation)?;
        Ok(SignalReadySnapshotOutput {
            reservation: self,
            outcome,
        })
    }
}

impl<D, I, T> SignalReadySnapshotOutput<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn into_outcome(
        self,
    ) -> SignalBranchSnapshotCaptureOutcome {
        self.reservation
            .admission
            .reach_operation_boundary(SignalOwnerOperationBoundary::OutcomeConstruction);
        let (branch_id, snapshot, observation) = self.outcome.into_output_parts();
        debug_assert_eq!(branch_id, self.reservation.branch_id);
        debug_assert_eq!(snapshot.meta.branch_id, branch_id);
        let mut retention = self.reservation.retention;
        let basis = self.reservation.owner.admit_canonical_basis(
            observation,
            branch_id,
            self.reservation.cell.incarnation().get(),
            retention.take_one(),
        );
        let snapshot = AdmittedSignalBranchSnapshot::owner_issued(
            self.reservation.owner.runtime_instance_id(),
            snapshot,
            retention.take_one(),
        );
        SignalBranchSnapshotCaptureOutcome::owner_issued(snapshot, basis)
    }
}

impl<'a, D, I, T> SignalRestoreOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn restore(
        self,
        expected: &AdmittedSignalBranchBasis,
        snapshot: &AdmittedSignalBranchSnapshot,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalReadyRestoreOutput<'a, D, I, T>, SignalBranchRestoreDenial> {
        self.restore_with_observers(expected, snapshot, cancellation, || {}, || {})
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn restore_with_cancellation_observers(
        self,
        expected: &AdmittedSignalBranchBasis,
        snapshot: &AdmittedSignalBranchSnapshot,
        cancellation: &SignalOwnerCancellationToken,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
    ) -> Result<SignalReadyRestoreOutput<'a, D, I, T>, SignalBranchRestoreDenial> {
        self.restore_with_observers(
            expected,
            snapshot,
            cancellation,
            before_movement,
            after_movement,
        )
    }

    fn restore_with_observers(
        self,
        expected: &AdmittedSignalBranchBasis,
        snapshot: &AdmittedSignalBranchSnapshot,
        cancellation: &SignalOwnerCancellationToken,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
    ) -> Result<SignalReadyRestoreOutput<'a, D, I, T>, SignalBranchRestoreDenial> {
        let snapshot_state = self
            .owner
            .metadata
            .snapshot_state(self.admission, snapshot)?
            .ok_or_else(|| SignalBranchRestoreDenial::UnavailableSnapshot {
                branch_id: snapshot.snapshot().meta.branch_id,
                snapshot_id: snapshot.snapshot().meta.snapshot_id,
            })?;
        let outcome = self.cell.restore_exact_with_observers(
            self.admission,
            expected,
            snapshot,
            snapshot_state,
            cancellation,
            before_movement,
            after_movement,
        )?;
        Ok(SignalReadyRestoreOutput {
            reservation: self,
            outcome,
        })
    }
}

impl<D, I, T> SignalReadyRestoreOutput<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn into_basis(self) -> AdmittedSignalBranchBasis {
        self.reservation
            .admission
            .reach_operation_boundary(SignalOwnerOperationBoundary::OutcomeConstruction);
        let (branch_id, observation) = self.outcome.into_output_parts();
        debug_assert_eq!(branch_id, self.reservation.branch_id);
        let mut retention = self.reservation.retention;
        self.reservation.owner.admit_canonical_basis(
            observation,
            branch_id,
            self.reservation.cell.incarnation().get(),
            retention.take_one(),
        )
    }
}

#[cfg(test)]
impl<'a, D, I, T> SignalForkOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn fork(
        mut self,
        source: &AdmittedSignalBranchBasis,
        requested_identity: ValidatedSignalBranchName,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalReadyForkOutput<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let destination =
            self.owner
                .reserve_fork_destination(self.admission, source, requested_identity)?;
        self.retention.rebind_all(destination.branch().id);
        let installed =
            destination.capture_and_install(Arc::clone(&self.cell), source, cancellation)?;
        Ok(SignalReadyForkOutput {
            reservation: self,
            installed,
        })
    }
}

#[cfg(test)]
impl<D, I, T> SignalReadyForkOutput<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn installed(
        &self,
    ) -> &SignalInstalledOwnerFork<'_, D, I, T> {
        &self.installed
    }

    pub(in crate::branch::owner_services) fn into_destination_parts(
        self,
    ) -> (SignalBranchHandle, AdmittedSignalBranchBasis) {
        let SignalReadyForkOutput {
            reservation,
            installed,
        } = self;
        reservation
            .admission
            .reach_operation_boundary(SignalOwnerOperationBoundary::OutcomeConstruction);
        let destination_cell_incarnation = installed.cell().incarnation().get();
        let (handle, observation) = installed.into_handoff_parts();
        let destination_branch_id = handle.id;
        debug_assert_ne!(reservation.source_branch_id, destination_branch_id);
        let mut retention = reservation.retention;
        let basis = reservation.owner.admit_canonical_basis(
            observation,
            destination_branch_id,
            destination_cell_incarnation,
            retention.take_one(),
        );
        (handle, basis)
    }
}
