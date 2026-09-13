use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::branch_execution_cell::SignalBranchCellAdmissionDenial;
use super::SignalOwnerRoot;

impl<D, I, T> SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn selected_branch_handle(&self) -> Option<SignalBranchHandle> {
        let owner = self.sealed_owner()?;
        self.branch_handle(owner.selected_branch_id())
    }

    pub(crate) fn known_branch_handles(&self) -> Vec<SignalBranchHandle> {
        let Some(owner) = self.sealed_owner() else {
            return Vec::new();
        };
        let Ok(admission) = owner.admit() else {
            return Vec::new();
        };
        let Ok(cells) = owner.registry.live_cells_in_identity_order(&admission) else {
            return Vec::new();
        };
        cells
            .into_iter()
            .filter_map(|cell| match cell.descriptive_handle(&admission) {
                Ok(handle) => Some(handle),
                Err(
                    SignalBranchCellAdmissionDenial::RetirementInProgress
                    | SignalBranchCellAdmissionDenial::RetiredIncarnation
                    | SignalBranchCellAdmissionDenial::PoisonedIncarnation,
                ) => None,
                Err(denial) => {
                    panic!("owner catalog inspection hit an invariant denial: {denial:?}")
                }
            })
            .collect()
    }

    pub(crate) fn branch_handle(&self, branch_id: SignalBranchId) -> Option<SignalBranchHandle> {
        let owner = self.sealed_owner()?;
        let admission = owner.admit().ok()?;
        owner
            .lookup_cell(&admission, branch_id)
            .ok()?
            .descriptive_handle(&admission)
            .ok()
    }

    pub(crate) fn branch_ancestry(&self, branch_id: SignalBranchId) -> Vec<SignalBranchHandle> {
        let Some(owner) = self.sealed_owner() else {
            return Vec::new();
        };
        let Ok(admission) = owner.admit() else {
            return Vec::new();
        };
        let mut ancestry = Vec::new();
        let mut cursor = Some(branch_id);
        while let Some(cursor_id) = cursor {
            let cell = match owner.lookup_cell(&admission, cursor_id) {
                Ok(cell) => cell,
                Err(
                    super::super::SignalBranchRegistryDenial::UnknownBranch(_)
                    | super::super::SignalBranchRegistryDenial::RetirementInProgress(_),
                ) => break,
                Err(denial) => {
                    panic!("owner ancestry inspection hit an invariant denial: {denial:?}")
                }
            };
            let handle = match cell.descriptive_handle(&admission) {
                Ok(handle) => handle,
                Err(
                    SignalBranchCellAdmissionDenial::RetirementInProgress
                    | SignalBranchCellAdmissionDenial::RetiredIncarnation
                    | SignalBranchCellAdmissionDenial::PoisonedIncarnation,
                ) => break,
                Err(denial) => {
                    panic!("owner ancestry inspection hit an invariant denial: {denial:?}")
                }
            };
            cursor = handle.parent_branch_id;
            ancestry.push(handle);
        }
        ancestry.reverse();
        ancestry
    }

    pub(crate) fn branch_head_snapshot_id(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<SignalSnapshotId> {
        self.branch_handle(branch_id)?.head_snapshot_id
    }
}
