use std::sync::Arc;

use crate::branch::SignalBranchForkOperationDenial;
use crate::state::SignalBranchId;

use super::super::lifecycle_state::SignalOwnerOperationAdmission;
use super::super::owner::SignalOwner;
use super::{SignalOwnerForkLineageReservation, SignalOwnerMetadata, SignalOwnerUnavailable};

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn reserve_fork_child_record(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        parent_branch_id: SignalBranchId,
        child_branch_id: SignalBranchId,
    ) -> Result<(), SignalBranchForkOperationDenial> {
        admission
            .authorize(self.runtime_instance_id, self.lifecycle_identity)
            .map_err(|_| {
                SignalBranchForkOperationDenial::OwnerUnavailable(SignalOwnerUnavailable)
            })?;
        let _hold = admission.hold_owner_metadata().map_err(|denial| match denial {
            super::super::lifecycle_state::SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld => {
                SignalBranchForkOperationDenial::OwnerCellMisuse {
                    branch_id: parent_branch_id,
                }
            }
            super::super::lifecycle_state::SignalOwnerMetadataHoldDenial::ExecutingThreadReentry => {
                SignalBranchForkOperationDenial::OwnerReentry
            }
        })?;
        let mut state = self.lock();
        if !state.fork_parent_accepts_child(parent_branch_id) {
            return Err(SignalBranchForkOperationDenial::RetirementInProgress {
                branch_id: parent_branch_id,
            });
        }
        state.record_fork_child(parent_branch_id, child_branch_id);
        Ok(())
    }
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_fork_child_owned(
        self: &Arc<Self>,
        admission: &SignalOwnerOperationAdmission<'_>,
        parent_branch_id: SignalBranchId,
        child_branch_id: SignalBranchId,
    ) -> Result<SignalOwnerOwnedForkLineageReservation<D, I, T>, SignalBranchForkOperationDenial>
    {
        self.metadata
            .reserve_fork_child_record(admission, parent_branch_id, child_branch_id)?;
        Ok(SignalOwnerOwnedForkLineageReservation {
            owner: Some(Arc::clone(self)),
            parent_branch_id,
            child_branch_id,
            committed: false,
        })
    }
}

pub(crate) struct SignalOwnerOwnedForkLineageReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Option<Arc<SignalOwner<D, I, T>>>,
    parent_branch_id: SignalBranchId,
    child_branch_id: SignalBranchId,
    committed: bool,
}

impl<D, I, T> SignalOwnerOwnedForkLineageReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn into_borrowed<'a>(
        mut self,
        metadata: &'a SignalOwnerMetadata<D, I, T>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
    ) -> SignalOwnerForkLineageReservation<'a, D, I, T> {
        let owner = self
            .owner
            .take()
            .expect("an owned fork lineage reservation has one owner");
        self.committed = true;
        drop(owner);
        SignalOwnerForkLineageReservation {
            metadata,
            admission,
            parent_branch_id: self.parent_branch_id,
            child_branch_id: self.child_branch_id,
            committed: false,
        }
    }
}

impl<D, I, T> Drop for SignalOwnerOwnedForkLineageReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Some(owner) = &self.owner {
            owner
                .metadata
                .lock()
                .remove_fork_child(self.parent_branch_id, self.child_branch_id);
        }
    }
}
