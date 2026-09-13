use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::branch::{SignalBranchForkOperationDenial, SignalBranchRetirementReceipt};
use crate::logic::transaction::SignalOwnerMetadataState;
use crate::state::SignalBranchId;

use super::lifecycle_state::SignalOwnerLifecycleIdentity;
use super::{SignalOwnerOperationAdmission, SignalOwnerUnavailable};

#[path = "owner_metadata/fork_lineage.rs"]
mod fork_lineage;
#[path = "owner_metadata/retirement.rs"]
mod retirement;
#[cfg(test)]
pub(in crate::branch::owner_services) use retirement::SignalOwnerRetirementContractObservation;
pub(super) use retirement::SignalOwnerRetirementMetadataReservation;
#[path = "owner_metadata/retention_acquisition.rs"]
mod retention_acquisition;
#[path = "owner_metadata/retirement_planning.rs"]
mod retirement_planning;
pub(super) use retention_acquisition::SignalOwnerRetentionAcquisitionDenial;
#[path = "owner_metadata/snapshot.rs"]
mod snapshot;
pub(super) use fork_lineage::SignalOwnerOwnedForkLineageReservation;
pub(crate) use snapshot::{SignalOwnerSnapshotReservation, SignalOwnerSnapshotStateBinding};

/// Short-lived owner metadata; canonical live branch truth never enters this lock.
pub(super) struct SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime_instance_id: u64,
    lifecycle_identity: SignalOwnerLifecycleIdentity,
    pending_snapshot_reservations: AtomicUsize,
    state: Mutex<SignalOwnerMetadataState<D, I, T>>,
}

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn new(
        state: SignalOwnerMetadataState<D, I, T>,
        runtime_instance_id: u64,
        lifecycle_identity: SignalOwnerLifecycleIdentity,
    ) -> Self {
        Self {
            runtime_instance_id,
            lifecycle_identity,
            pending_snapshot_reservations: AtomicUsize::new(0),
            state: Mutex::new(state),
        }
    }

    pub(super) fn reserve_fork_child<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        parent_branch_id: SignalBranchId,
        child_branch_id: SignalBranchId,
    ) -> Result<SignalOwnerForkLineageReservation<'a, D, I, T>, SignalBranchForkOperationDenial>
    {
        self.reserve_fork_child_record(admission, parent_branch_id, child_branch_id)?;
        Ok(SignalOwnerForkLineageReservation {
            metadata: self,
            admission,
            parent_branch_id,
            child_branch_id,
            committed: false,
        })
    }

    pub(super) fn branch_children(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<Vec<SignalBranchId>, SignalOwnerMetadataAuthorizationDenial> {
        let _hold = self.authorize(admission)?;
        Ok(self.lock().branch_children(branch_id))
    }

    pub(super) fn is_merge_participant(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<bool, SignalOwnerMetadataAuthorizationDenial> {
        let _hold = self.authorize(admission)?;
        Ok(self.lock().is_merge_participant(branch_id))
    }

    pub(super) fn retirement_receipt(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<Option<SignalBranchRetirementReceipt>, SignalOwnerMetadataAuthorizationDenial> {
        let _hold = self.authorize(admission)?;
        Ok(self.lock().branch_retirement_receipt(branch_id))
    }

    pub(super) fn branch_accepts_retention_acquisition(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<bool, SignalOwnerMetadataAuthorizationDenial> {
        let _hold = self.authorize(admission)?;
        Ok(self.lock().branch_accepts_retention_acquisition(branch_id))
    }

    pub(super) fn take_close_batch(
        &self,
        maximum_batch_size: usize,
    ) -> crate::logic::transaction::SignalOwnerMetadataCloseBatch<D, I, T> {
        debug_assert_eq!(
            self.pending_snapshot_reservations.load(Ordering::Acquire),
            0
        );
        self.lock().take_close_batch(maximum_batch_size)
    }

    fn lock(&self) -> MutexGuard<'_, SignalOwnerMetadataState<D, I, T>> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn authorize<'a>(
        &self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
    ) -> Result<
        super::lifecycle_state::SignalOwnerMetadataHold<'a>,
        SignalOwnerMetadataAuthorizationDenial,
    > {
        admission
            .authorize(self.runtime_instance_id, self.lifecycle_identity)
            .map_err(|_| SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable)?;
        admission.hold_owner_metadata().map_err(|denial| match denial {
            super::lifecycle_state::SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld => {
                SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse
            }
            super::lifecycle_state::SignalOwnerMetadataHoldDenial::ExecutingThreadReentry => {
                SignalOwnerMetadataAuthorizationDenial::OwnerReentry
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SignalOwnerMetadataAuthorizationDenial {
    OwnerUnavailable,
    OwnerCellMisuse,
    OwnerReentry,
}

pub(super) struct SignalOwnerForkLineageReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    metadata: &'a SignalOwnerMetadata<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    parent_branch_id: SignalBranchId,
    child_branch_id: SignalBranchId,
    committed: bool,
}

impl<D, I, T> SignalOwnerForkLineageReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn commit(mut self) {
        self.committed = true;
    }
}

impl<D, I, T> Drop for SignalOwnerForkLineageReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if !self.committed {
            debug_assert!(
                self.admission.permits_owner_lock_acquisition(),
                "fork lineage cleanup must run after target-cell release"
            );
            self.metadata
                .lock()
                .remove_fork_child(self.parent_branch_id, self.child_branch_id);
        }
    }
}
