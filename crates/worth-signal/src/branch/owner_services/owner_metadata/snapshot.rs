use std::sync::atomic::Ordering;

use crate::branch::{
    AdmittedSignalBranchSnapshot, SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};
use crate::logic::transaction::{
    SignalOwnerSnapshotReservationDenial, SnapshotBranchState, SnapshotStatePacket,
};
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::super::{SignalBranchCellState, SignalBranchExecutionCell};
use super::{
    SignalOwnerMetadata, SignalOwnerMetadataAuthorizationDenial, SignalOwnerOperationAdmission,
    SignalOwnerUnavailable,
};
use crate::branch::owner_services::cell_incarnation::SignalBranchCellIncarnation;
use crate::branch::owner_services::lifecycle_state::SignalOwnerLifecycleIdentity;

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_snapshot<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        cell: &SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>,
    ) -> Result<SignalOwnerSnapshotReservation<'a, D, I, T>, SignalBranchSnapshotCaptureDenial>
    {
        let branch_id = cell.branch_id();
        cell.validate_admission(admission).map_err(|denial| {
            super::super::branch_execution_cell::snapshot::map_snapshot_cell_denial(
                denial, branch_id,
            )
        })?;
        let _hold = self
            .authorize(admission)
            .map_err(|denial| map_snapshot_authorization_denial(denial, branch_id))?;
        let mut state = self.lock();
        let pending = self.pending_snapshot_reservations.load(Ordering::Acquire);
        let snapshot_id = state
            .reserve_snapshot(pending)
            .map_err(|denial| match denial {
                SignalOwnerSnapshotReservationDenial::CapacityExhausted {
                    maximum_stored_snapshots,
                } => SignalBranchSnapshotCaptureDenial::SnapshotCapacityExhausted {
                    maximum_stored_snapshots,
                },
                SignalOwnerSnapshotReservationDenial::IdentityExhausted { next_snapshot_id } => {
                    SignalBranchSnapshotCaptureDenial::SnapshotIdentityExhausted {
                        next_snapshot_id,
                    }
                }
            })?;
        self.pending_snapshot_reservations
            .fetch_add(1, Ordering::AcqRel);
        drop(state);
        Ok(SignalOwnerSnapshotReservation {
            metadata: self,
            admission,
            branch_id,
            cell_incarnation: cell.incarnation(),
            snapshot_id,
            installed: false,
        })
    }

    pub(in crate::branch::owner_services) fn snapshot_state(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        admitted_snapshot: &AdmittedSignalBranchSnapshot,
    ) -> Result<Option<SignalOwnerSnapshotStateBinding<D, I, T>>, SignalBranchRestoreDenial> {
        let snapshot = admitted_snapshot.snapshot();
        let _hold = self
            .authorize(admission)
            .map_err(|denial| map_restore_authorization_denial(denial, snapshot.meta.branch_id))?;
        if admitted_snapshot.owner_runtime_instance_id() != self.runtime_instance_id {
            return Err(SignalBranchRestoreDenial::ForeignSnapshotOwner {
                expected_runtime_instance_id: self.runtime_instance_id,
                observed_runtime_instance_id: admitted_snapshot.owner_runtime_instance_id(),
            });
        }
        Ok(self
            .lock()
            .snapshot_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id)
            .map(|state| SignalOwnerSnapshotStateBinding {
                runtime_instance_id: self.runtime_instance_id,
                lifecycle_identity: self.lifecycle_identity,
                branch_id: snapshot.meta.branch_id,
                snapshot_id: snapshot.meta.snapshot_id,
                state,
            }))
    }

    fn release_snapshot_reservation(&self) {
        let prior = self
            .pending_snapshot_reservations
            .fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prior > 0, "snapshot reservation releases exactly once");
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn pending_snapshot_reservation_count(&self) -> usize {
        self.pending_snapshot_reservations.load(Ordering::Acquire)
    }

    pub(in crate::branch::owner_services) fn has_snapshot_state(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    ) -> Result<bool, SignalOwnerMetadataAuthorizationDenial> {
        let _hold = self.authorize(admission)?;
        Ok(self.lock().snapshot_state(branch_id, snapshot_id).is_some())
    }
}

fn map_snapshot_authorization_denial(
    denial: SignalOwnerMetadataAuthorizationDenial,
    branch_id: SignalBranchId,
) -> SignalBranchSnapshotCaptureDenial {
    match denial {
        SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable => {
            SignalBranchSnapshotCaptureDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse => {
            SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id }
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerReentry => {
            SignalBranchSnapshotCaptureDenial::OwnerReentry
        }
    }
}

fn map_restore_authorization_denial(
    denial: SignalOwnerMetadataAuthorizationDenial,
    branch_id: SignalBranchId,
) -> SignalBranchRestoreDenial {
    match denial {
        SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable => {
            SignalBranchRestoreDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse => {
            SignalBranchRestoreDenial::OwnerCellMisuse { branch_id }
        }
        SignalOwnerMetadataAuthorizationDenial::OwnerReentry => {
            SignalBranchRestoreDenial::OwnerReentry
        }
    }
}

pub(crate) struct SignalOwnerSnapshotStateBinding<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime_instance_id: u64,
    lifecycle_identity: SignalOwnerLifecycleIdentity,
    branch_id: SignalBranchId,
    snapshot_id: SignalSnapshotId,
    state: SnapshotBranchState<D, I, T>,
}

impl<D, I, T> SignalOwnerSnapshotStateBinding<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch) fn into_state_for(
        self,
        runtime_instance_id: u64,
        lifecycle_identity: SignalOwnerLifecycleIdentity,
        admitted_snapshot: &AdmittedSignalBranchSnapshot,
    ) -> Result<SnapshotBranchState<D, I, T>, SignalBranchRestoreDenial> {
        if self.runtime_instance_id != runtime_instance_id
            || self.lifecycle_identity != lifecycle_identity
        {
            return Err(SignalBranchRestoreDenial::OwnerUnavailable(
                SignalOwnerUnavailable,
            ));
        }
        let snapshot = admitted_snapshot.snapshot();
        if admitted_snapshot.owner_runtime_instance_id() != runtime_instance_id {
            return Err(SignalBranchRestoreDenial::ForeignSnapshotOwner {
                expected_runtime_instance_id: runtime_instance_id,
                observed_runtime_instance_id: admitted_snapshot.owner_runtime_instance_id(),
            });
        }
        if self.branch_id != snapshot.meta.branch_id
            || self.snapshot_id != snapshot.meta.snapshot_id
        {
            return Err(SignalBranchRestoreDenial::UnavailableSnapshot {
                branch_id: snapshot.meta.branch_id,
                snapshot_id: snapshot.meta.snapshot_id,
            });
        }
        Ok(self.state)
    }
}

pub(crate) struct SignalOwnerSnapshotReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    metadata: &'a SignalOwnerMetadata<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    branch_id: SignalBranchId,
    cell_incarnation: SignalBranchCellIncarnation,
    snapshot_id: SignalSnapshotId,
    installed: bool,
}

impl<'a, D, I, T> SignalOwnerSnapshotReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn admission(
        &self,
    ) -> &'a SignalOwnerOperationAdmission<'a> {
        self.admission
    }

    pub(in crate::branch::owner_services) const fn snapshot_id(&self) -> SignalSnapshotId {
        self.snapshot_id
    }

    pub(in crate::branch::owner_services) fn matches_cell(
        &self,
        cell: &SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>,
    ) -> bool {
        self.branch_id == cell.branch_id() && self.cell_incarnation == cell.incarnation()
    }

    pub(crate) fn install(mut self, packet: SnapshotStatePacket<D, I, T>) {
        assert!(
            self.admission.permits_owner_lock_acquisition(),
            "snapshot installation must run after target-cell release"
        );
        self.metadata
            .lock()
            .install_reserved_snapshot(self.snapshot_id, packet);
        self.metadata.release_snapshot_reservation();
        self.installed = true;
    }
}

impl<D, I, T> Drop for SignalOwnerSnapshotReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if !self.installed {
            let ordering_is_valid = self.admission.permits_owner_lock_acquisition();
            self.metadata.release_snapshot_reservation();
            if !ordering_is_valid && !std::thread::panicking() {
                panic!("snapshot reservation cleanup must run after target-cell release");
            }
        }
    }
}
